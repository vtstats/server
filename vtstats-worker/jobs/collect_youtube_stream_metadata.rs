use chrono::{DateTime, Duration, DurationRound, NaiveDateTime, Utc};
use integration_youtube::{
    data_api::videos::{list_videos, Stream},
    youtubei::{
        live_chat::{LiveChatMessage, MemberMessageType, PaidMessageType},
        updated_metadata, updated_metadata_with_continuation, youtube_live_chat,
        youtube_live_chat_with_continuation,
    },
};
use vtstats_database::{
    jobs::{queue_send_notification, CollectYoutubeStreamMetadataJobPayload},
    stream_events::{
        add_stream_events, StreamEventValue, YoutubeMemberMilestoneDonationValue,
        YoutubeNewMemberDonationValue, YoutubeSuperChatDonationValue,
        YoutubeSuperStickerDonationValue,
    },
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow, AddStreamViewerStatsQuery},
    streams::{
        delete_stream, end_stream, end_stream_with_values, find_stream, start_stream, StreamStatus,
    },
    PgPool,
};
use vtstats_request::RequestHub;

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    hub: RequestHub,
    continuation: Option<String>,
    payload: CollectYoutubeStreamMetadataJobPayload,
) -> anyhow::Result<JobResult> {
    let CollectYoutubeStreamMetadataJobPayload {
        stream_id,
        platform_stream_id,
        platform_channel_id,
    } = payload;

    let Some(find) = find_stream(stream_id, pool).await? else {
        return Ok(JobResult::Completed);
    };

    let (metadata_continuation, chat_continuation) = continuation
        .and_then(|c| serde_json::from_str::<(String, String)>(&c).ok())
        .unwrap_or_default();

    let metadata = if !metadata_continuation.is_empty() {
        updated_metadata_with_continuation(&metadata_continuation, &hub.client).await
    } else {
        updated_metadata(&platform_stream_id, &hub.client).await
    }?;

    let (mut timeout, next_metadata_continuation) =
        match (metadata.timeout(), metadata.continuation()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                // stream not found
                if find.status == StreamStatus::Scheduled {
                    delete_stream(stream_id, pool).await?;
                } else {
                    end_stream(stream_id, pool).await?;
                }
                return Ok(JobResult::Completed);
            }
        };

    // stream is still waiting
    if metadata.is_waiting() {
        let continuation = serde_json::to_string(&(next_metadata_continuation, "")).ok();

        return Ok(JobResult::Next {
            run: Utc::now() + Duration::from_std(timeout)?,
            continuation,
        });
    }

    let (messages, next_chat_timeout, next_chat_continuation) = if !chat_continuation.is_empty() {
        youtube_live_chat_with_continuation(chat_continuation, &hub.client).await
    } else {
        youtube_live_chat(&platform_channel_id, &platform_stream_id, &hub.client).await
    }
    .map(|(messages, continuation)| {
        match continuation.and_then(|c| c.get_continuation_and_timeout()) {
            Some((t, c)) => (messages, Some(t), Some(c)),
            _ => (messages, None, None),
        }
    })?;

    let _ = collect_donation_and_chat(stream_id, messages, pool).await;

    let now = Utc::now();

    if let Some(viewer) = metadata.view_count() {
        AddStreamViewerStatsQuery {
            time: now.duration_trunc(Duration::seconds(15)).unwrap(),
            count: viewer,
            stream_id,
        }
        .execute(pool)
        .await?;
    }

    // stream has ended
    if timeout.as_secs() > 5 {
        if find.status == StreamStatus::Scheduled {
            delete_stream(stream_id, pool).await?;
            return Ok(JobResult::Completed);
        }

        let mut videos = list_videos(&platform_stream_id, &hub.client).await?;
        let stream: Option<Stream> = videos.pop().and_then(Into::into);

        end_stream_with_values(
            stream_id,
            stream.as_ref().map(|s| s.title.as_str()),
            stream.as_ref().and_then(|s| s.schedule_time),
            stream.as_ref().and_then(|s| s.start_time),
            stream.as_ref().and_then(|s| s.end_time),
            stream.as_ref().and_then(|s| s.likes),
            pool,
        )
        .await?;

        queue_send_notification(
            now,
            "youtube".into(),
            platform_stream_id,
            find.vtuber_id,
            pool,
        )
        .await?;

        return Ok(JobResult::Completed);
    }

    start_stream(
        stream_id,
        metadata.title().as_deref(),
        now,
        metadata.like_count(),
        pool,
    )
    .await?;

    let continuation = serde_json::to_string(&(
        next_metadata_continuation,
        next_chat_continuation.unwrap_or_default(),
    ))
    .ok();

    if let Some(chat_timeout) = next_chat_timeout {
        timeout = std::cmp::max(timeout, chat_timeout);
    }

    Ok(JobResult::Next {
        run: now + Duration::from_std(timeout)?,
        continuation,
    })
}

async fn collect_donation_and_chat(
    stream_id: i32,
    messages: Vec<LiveChatMessage>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }

    let mut donation_rows = Vec::<(DateTime<Utc>, StreamEventValue)>::new();

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

                let time = time.duration_trunc(Duration::seconds(15)).unwrap();

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
                        StreamEventValue::YoutubeNewMember(YoutubeNewMemberDonationValue {
                            message: text,
                            author_name,
                            author_badges: (!badges.is_empty()).then_some(badges),
                            author_channel_id,
                        })
                    }
                    MemberMessageType::Milestone => StreamEventValue::YoutubeMemberMilestone(
                        YoutubeMemberMilestoneDonationValue {
                            author_name,
                            author_badges: (!badges.is_empty()).then_some(badges),
                            author_channel_id,
                        },
                    ),
                };

                donation_rows.push((time, value))
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
                        StreamEventValue::YoutubeSuperChat(YoutubeSuperChatDonationValue {
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
                        StreamEventValue::YoutubeSuperSticker(YoutubeSuperStickerDonationValue {
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

                donation_rows.push((time, value))
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

    if !donation_rows.is_empty() {
        add_stream_events(stream_id, donation_rows, pool).await?;
    }

    Ok(())
}

fn parse_timestamp(string: &str) -> Option<DateTime<Utc>> {
    if string.len() <= 6 {
        return None;
    }

    let (secs, nsecs) = string.split_at(string.len() - 6);

    Some(DateTime::from_utc(
        NaiveDateTime::from_timestamp_opt(secs.parse().ok()?, nsecs.parse().ok()?)?,
        Utc,
    ))
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
        Some(DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(1620129525, 0).unwrap(),
            Utc,
        ))
    );
    assert_eq!(
        parse_timestamp("1620129525606474"),
        Some(DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(1620129525, 606474).unwrap(),
            Utc,
        ))
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
