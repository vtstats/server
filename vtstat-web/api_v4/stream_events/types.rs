use serde::Serialize;
use vtstat_database::stream_events::StreamEventValue;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum RefinedStreamEventValue {
    YoutubeSuperChat {
        amount: String,
        currency_code: String,
        color: YouTubeChatColor,
    },
    YoutubeSuperSticker {
        amount: String,
        currency_code: String,
        color: YouTubeChatColor,
    },
    YoutubeNewMember,
    YoutubeMemberMilestone,
}

pub fn refine(value: StreamEventValue) -> Option<RefinedStreamEventValue> {
    match value {
        StreamEventValue::YoutubeSuperChat(v) => Some(RefinedStreamEventValue::YoutubeSuperChat {
            amount: v.paid_amount,
            currency_code: currency_symbol_to_code(&v.paid_currency_symbol)?.into(),
            color: color_hex_to_chat_color(&v.paid_color)?,
        }),
        StreamEventValue::YoutubeSuperSticker(v) => {
            Some(RefinedStreamEventValue::YoutubeSuperSticker {
                amount: v.paid_amount,
                currency_code: currency_symbol_to_code(&v.paid_currency_symbol)?.into(),
                color: color_hex_to_chat_color(&v.paid_color)?,
            })
        }
        StreamEventValue::YoutubeNewMember(_) => Some(RefinedStreamEventValue::YoutubeNewMember),
        StreamEventValue::YoutubeMemberMilestone(_) => {
            Some(RefinedStreamEventValue::YoutubeMemberMilestone)
        }
    }
}

fn currency_symbol_to_code(i: &str) -> Option<&str> {
    match i {
        "$" => Some("USD"),
        "€" => Some("EUR"),
        "¥" | "¥," => Some("JPY"),
        "£" => Some("GBP"),
        "A$" => Some("AUD"),
        "CA$" => Some("CAD"),
        "HK$" => Some("HKD"),
        "NZ$" => Some("NZD"),
        "₩" => Some("KRW"),
        "MX$" => Some("MXN"),
        "₹" => Some("INR"),
        "R$" => Some("BRL"),
        "NT$" => Some("TWD"),
        "₪" => Some("ILS"),
        "₱" => Some("PHP"),
        i if i.len() == 3 && i.bytes().all(|c| c.is_ascii_uppercase()) => Some(i),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YouTubeChatColor {
    Green,
    Yellow,
    Blue,
    LightBlue,
    Orange,
    Magenta,
    Red,
}

fn color_hex_to_chat_color(i: &str) -> Option<YouTubeChatColor> {
    match i {
        "1DE9B6FF" => Some(YouTubeChatColor::Green),
        "FFCA28FF" => Some(YouTubeChatColor::Yellow),
        "1E88E5FF" => Some(YouTubeChatColor::Blue),
        "00E5FFFF" => Some(YouTubeChatColor::LightBlue),
        "F57C00FF" => Some(YouTubeChatColor::Orange),
        "E91E63FF" => Some(YouTubeChatColor::Magenta),
        "E62117FF" => Some(YouTubeChatColor::Red),
        _ => None,
    }
}
