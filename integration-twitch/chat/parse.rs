use chrono::{DateTime, NaiveDateTime, Utc};
use twitch_message::messages::Privmsg;

#[derive(Debug)]
pub enum LiveChatMessage {
    Text {
        author_username: String,
        timestamp: DateTime<Utc>,
        text: String,
        badges: Option<String>,
    },
    Subscriber {
        author_username: String,
        timestamp: DateTime<Utc>,
        text: String,
        badges: Option<String>,
    },
    HyperChat {
        author_username: String,
        timestamp: DateTime<Utc>,
        text: String,
        badges: Option<String>,
        currency: String,
        amount: String,
        level: String,
    },
}

pub fn parse_privmsg(msg: Privmsg<'_>) -> Option<LiveChatMessage> {
    // https://dev.twitch.tv/docs/irc/tags/#privmsg-tags
    let badges = msg
        .tags
        .get("badge-info")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let timestamp = msg.tmi_sent_ts()?;
    let timestamp = DateTime::from_utc(
        NaiveDateTime::from_timestamp_millis(timestamp.parse().ok()?)?,
        Utc,
    );
    let text = msg.data.to_string();
    let author_username = msg.sender.to_string();

    if let (Some(amount), Some(currency), Some(level)) = (
        msg.tags.get("pinned-chat-paid-amount"),
        msg.tags.get("pinned-chat-paid-currency"),
        msg.tags.get("pinned-chat-paid-level"),
    ) {
        Some(LiveChatMessage::HyperChat {
            author_username,
            badges,
            text,
            timestamp,
            amount: amount.to_string(),
            currency: currency.to_string(),
            level: level.to_string(),
        })
    } else if matches!(&badges, Some(b) if b.contains("subscriber/")) {
        Some(LiveChatMessage::Subscriber {
            author_username,
            badges,
            text,
            timestamp,
        })
    } else {
        Some(LiveChatMessage::Text {
            author_username,
            badges,
            text,
            timestamp,
        })
    }
}

#[test]
fn test_parse() {
    use twitch_message::parse;

    let msg = "@badge-info=;badges=;bits=100;color=#00FF7F;display-name=TonyPeppermint;emotes=;first-msg=0;flags=;id=a7944867-e77d-4a39-a857-e8fb6f2aa03b;mod=0;returning-chatter=0;room-id=175831187;subscriber=0;tmi-sent-ts=1693940568571;turbo=0;user-id=185324437;user-type= :tonypeppermint!tonypeppermint@tonypeppermint.tmi.twitch.tv PRIVMSG #ironmouse :Cheer100 Barry with the Hard Drip.";
    let msg = parse_privmsg(Privmsg::try_from(parse(msg).unwrap().message).unwrap()).unwrap();
    assert!(matches!(msg, LiveChatMessage::Text { .. }));
}
