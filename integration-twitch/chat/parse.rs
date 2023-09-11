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
    Cheering {
        bits: String,
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
        currency_code: String,
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

    if let Some(bits) = msg.tags.get("bits") {
        Some(LiveChatMessage::Cheering {
            author_username,
            badges,
            text,
            timestamp,
            bits: bits.to_string(),
        })
    } else if let (Some(amount), Some(currency_code), Some(level)) = (
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
            currency_code: currency_code.to_string(),
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
    assert!(matches!(msg, LiveChatMessage::Cheering { .. }));

    let msg = "@badge-info=subscriber/28;badges=subscriber/24,premium/1;client-nonce=49666e39d55d238078e09de8c9b38fd9;color=#FF0000;display-name=Lark88;emotes=;first-msg=0;flags=;id=049908f8-b65f-4627-985e-e8808acdf378;mod=0;returning-chatter=0;room-id=175831187;subscriber=1;tmi-sent-ts=1693940565428;turbo=0;user-id=93894180;user-type= :lark88!lark88@lark88.tmi.twitch.tv PRIVMSG #ironmouse :Best boi!";
    let msg = parse_privmsg(Privmsg::try_from(parse(msg).unwrap().message).unwrap()).unwrap();
    assert!(matches!(msg, LiveChatMessage::Subscriber { .. }));

    let msg = "@badge-info=;badges=;color=;display-name=themightyspoon1;emotes=emotesv2_ad00a34a54084c56ae1698542b5b6f53:6-18/emotesv2_dcd06b30a5c24f6eb871e8f5edbd44f7:20-28;first-msg=0;flags=;id=ec894756-d624-470f-9c3c-8a86ea3c3e9c;mod=0;returning-chatter=0;room-id=175831187;subscriber=0;tmi-sent-ts=1693940586209;turbo=0;user-id=951421495;user-type= :themightyspoon1!themightyspoon1@themightyspoon1.tmi.twitch.tv PRIVMSG #ironmouse :BARRY ironmouseRAVE DinoDance";
    let msg = parse_privmsg(Privmsg::try_from(parse(msg).unwrap().message).unwrap()).unwrap();
    assert!(matches!(msg, LiveChatMessage::Text { .. }));

    let msg = "@badge-info=;badges=glhf-pledge/1;color=;emotes=;first-msg=0;flags=;id=f6fb34f8-562f-4b4d-b628-32113d0ef4b0;mod=0;pinned-chat-paid-amount=200;pinned-chat-paid-canonical-amount=200;pinned-chat-paid-currency=USD;pinned-chat-paid-exponent=2;pinned-chat-paid-is-system-message=0;pinned-chat-paid-level=ONE;returning-chatter=0;room-id=12345678;subscriber=0;tmi-sent-ts=1687471984306;turbo=0;user-id=12345678;user-type= :abc!abc@abc.tmi.twitch.tv PRIVMSG #xyz :HeyGuys";
    let msg = parse_privmsg(Privmsg::try_from(parse(msg).unwrap().message).unwrap()).unwrap();
    assert!(matches!(msg, LiveChatMessage::HyperChat { .. }));
}
