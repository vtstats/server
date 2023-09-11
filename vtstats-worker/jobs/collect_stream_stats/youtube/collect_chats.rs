use chrono::{DateTime, DurationRound, TimeZone, Utc};
use integration_youtube::youtubei::{
    youtube_live_chat, youtube_live_chat_with_continuation, LiveChatMessage, MemberMessageType,
    PaidMessageType,
};
use reqwest::Client;
use std::time::Duration;
use vtstats_database::{
    channels::Channel,
    stream_events::{
        add_stream_events, StreamEventValue, YoutubeMemberMilestone, YoutubeNewMember,
        YoutubeSuperChat, YoutubeSuperSticker,
    },
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow},
    streams::Stream,
    PgPool,
};

pub async fn collect_chats(
    channel: &Channel,
    stream: &Stream,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut continuation: Option<String> = None;
    let mut timeout = Duration::from_secs(15);

    loop {
        let messages;
        if let Some(c) = continuation.take() {
            let res = youtube_live_chat_with_continuation(c, client).await?;
            messages = res.0;
            if let Some((next_timeout, next_continuation)) =
                res.1.and_then(|c| c.get_continuation_and_timeout())
            {
                timeout = next_timeout;
                continuation = Some(next_continuation);
            }
        } else {
            let res = youtube_live_chat(&channel.platform_id, &stream.platform_id, client).await?;
            messages = res.0;
            if let Some((next_timeout, next_continuation)) =
                res.1.and_then(|c| c.get_continuation_and_timeout())
            {
                timeout = next_timeout;
                continuation = Some(next_continuation);
            }
        }

        let _ = collect_chat_and_events(stream.stream_id, messages, pool).await;

        tokio::time::sleep(timeout).await;
    }
}

async fn collect_chat_and_events(
    stream_id: i32,
    messages: Vec<LiveChatMessage>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }

    let mut stream_event_rows = Vec::<(DateTime<Utc>, StreamEventValue)>::new();

    let mut chat_stats_rows = Vec::<AddStreamChatStatsRow>::new();

    for message in messages {
        match message {
            LiveChatMessage::Text {
                timestamp, badges, ..
            } => {
                let Some(time) = parse_timestamp(&timestamp) else {
                    tracing::warn!("Failed to parse timestamp: {timestamp:?}.");
                    continue;
                };

                let time = time.duration_trunc(chrono::Duration::seconds(15)).unwrap();

                let from_member = badges
                    .iter()
                    .any(|badge| badge.contains("Member") || badge.contains("member"));

                if let Some(row) = chat_stats_rows.iter_mut().find(|row| row.time == time) {
                    row.count += 1;
                    row.from_member_count += if from_member { 1 } else { 0 };
                } else {
                    chat_stats_rows.push(AddStreamChatStatsRow {
                        time,
                        count: 1,
                        from_member_count: if from_member { 1 } else { 0 },
                    })
                }
            }

            LiveChatMessage::Member {
                ty,
                author_name,
                author_channel_id,
                timestamp,
                text,
                badges,
            } => {
                let Some(time) = parse_timestamp(&timestamp) else {
                    tracing::warn!("Failed to parse timestamp: {timestamp:?}.");
                    continue;
                };

                let badges = badges.join(",");

                let value = match ty {
                    MemberMessageType::New => {
                        StreamEventValue::YoutubeNewMember(YoutubeNewMember {
                            message: text,
                            author_name,
                            author_badges: (!badges.is_empty()).then_some(badges),
                            author_channel_id,
                        })
                    }
                    MemberMessageType::Milestone => {
                        StreamEventValue::YoutubeMemberMilestone(YoutubeMemberMilestone {
                            author_name,
                            author_badges: (!badges.is_empty()).then_some(badges),
                            author_channel_id,
                        })
                    }
                };

                stream_event_rows.push((time, value))
            }

            LiveChatMessage::Paid {
                ty,
                author_name,
                author_channel_id,
                timestamp,
                amount,
                badges,
                text,
                color,
            } => {
                let Some(time) = parse_timestamp(&timestamp) else {
                    tracing::warn!("Failed to parse timestamp: {timestamp:?}.");
                    continue;
                };

                let Some((paid_symbol, paid_amount)) = parse_amount(&amount) else {
                    tracing::warn!("Failed to parse amount: {amount:?}.");
                    continue;
                };

                let badges = badges.join(",");

                let value = match ty {
                    PaidMessageType::SuperChat => {
                        StreamEventValue::YoutubeSuperChat(YoutubeSuperChat {
                            paid_amount,
                            paid_currency_symbol: paid_symbol.into(),
                            paid_color: color,
                            message: (!text.is_empty()).then_some(text),
                            author_name,
                            author_badges: (!badges.is_empty()).then_some(badges),
                            author_channel_id,
                        })
                    }
                    PaidMessageType::SuperSticker => {
                        StreamEventValue::YoutubeSuperSticker(YoutubeSuperSticker {
                            paid_amount,
                            paid_currency_symbol: paid_symbol.into(),
                            paid_color: color,
                            message: (!text.is_empty()).then_some(text),
                            author_name,
                            author_badges: (!badges.is_empty()).then_some(badges),
                            author_channel_id,
                        })
                    }
                };

                stream_event_rows.push((time, value))
            }
        }
    }

    if !chat_stats_rows.is_empty() {
        AddStreamChatStatsQuery {
            stream_id,
            rows: chat_stats_rows,
        }
        .execute(pool)
        .await?;
    }

    if !stream_event_rows.is_empty() {
        add_stream_events(stream_id, stream_event_rows, pool).await?;
    }

    Ok(())
}

pub fn parse_timestamp(string: &str) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(string.parse().ok()?).single()
}

fn parse_amount(amount: &str) -> Option<(&str, String)> {
    let i = amount.find(|ch: char| ch.is_ascii_digit())?;

    let (mut symbol, mut value) = amount.split_at(i);

    symbol = symbol.trim();
    value = value.trim();

    if symbol.is_empty() || value.is_empty() {
        return None;
    }

    let value = value.replace(',', "");

    let _: f32 = value.parse().ok()?;

    Some((symbol, value))
}

#[test]
fn test_parse_timestamp() {
    assert_eq!(parse_timestamp(""), None);
    assert_eq!(parse_timestamp("                "), None);
    assert_eq!(parse_timestamp("XXXXXXXXXXXXXXXX"), None);
    assert_eq!(
        parse_timestamp("1620129525000000"),
        Some(Utc.timestamp_millis_opt(1620129525000000).single().unwrap())
    );
    assert_eq!(
        parse_timestamp("1620129525606474"),
        Some(Utc.timestamp_millis_opt(1620129525606474).single().unwrap())
    );
}

#[test]
fn test_parse_amount() {
    assert_eq!(parse_amount("",), None);
    assert_eq!(parse_amount("$",), None);
    assert_eq!(parse_amount("¥¥",), None);
    assert_eq!(parse_amount("JPY",), None);
    assert_eq!(parse_amount("10.00",), None);
    assert_eq!(parse_amount("1.0.00",), None);
    assert_eq!(parse_amount("$10.0.00",), None);
    assert_eq!(parse_amount("$1,000.00",), Some(("$", "1000.00".into())));
    assert_eq!(parse_amount("JPY 99.99",), Some(("JPY", "99.99".into())));
    assert_eq!(parse_amount("¥99.9999",), Some(("¥", "99.9999".into())));
    assert_eq!(
        parse_amount("USD 99.9901",),
        Some(("USD", "99.9901".into()))
    );
}
