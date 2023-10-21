use serde::Serialize;
use vtstats_database::stream_events::StreamEventValue;
use vtstats_utils::currency::currency_symbol_to_code;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum RefinedStreamEventValue {
    #[serde(rename_all = "camelCase")]
    YouTubeSuperChat {
        amount: String,
        currency_code: String,
        color: YouTubeChatColor,
    },
    #[serde(rename_all = "camelCase")]
    YouTubeSuperSticker {
        amount: String,
        currency_code: String,
        color: YouTubeChatColor,
    },
    YouTubeNewMember,
    YouTubeMemberMilestone,
    TwitchCheering {
        bits: usize,
    },
    TwitchHyperChat {
        amount: String,
        currency_code: String,
    },
}

impl RefinedStreamEventValue {
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(
            self,
            RefinedStreamEventValue::YouTubeMemberMilestone
                | RefinedStreamEventValue::YouTubeNewMember
        )
    }
}

pub fn refine(value: StreamEventValue) -> Option<RefinedStreamEventValue> {
    match value {
        StreamEventValue::YoutubeSuperChat(v) => Some(RefinedStreamEventValue::YouTubeSuperChat {
            amount: v.paid_amount,
            currency_code: currency_symbol_to_code(&v.paid_currency_symbol)?.into(),
            color: color_hex_to_chat_color(&v.paid_color)?,
        }),
        StreamEventValue::YoutubeSuperSticker(v) => {
            Some(RefinedStreamEventValue::YouTubeSuperSticker {
                amount: v.paid_amount,
                currency_code: currency_symbol_to_code(&v.paid_currency_symbol)?.into(),
                color: color_hex_to_chat_color(&v.paid_color)?,
            })
        }
        StreamEventValue::YoutubeNewMember(_) => Some(RefinedStreamEventValue::YouTubeNewMember),
        StreamEventValue::YoutubeMemberMilestone(_) => {
            Some(RefinedStreamEventValue::YouTubeMemberMilestone)
        }
        StreamEventValue::TwitchCheering(v) => Some(RefinedStreamEventValue::TwitchCheering {
            bits: v.bits.parse().ok()?,
        }),
        StreamEventValue::TwitchHyperChat(v) => Some(RefinedStreamEventValue::TwitchHyperChat {
            amount: v.amount,
            currency_code: v.currency_code,
        }),
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
