use chrono::{DateTime, Duration, DurationRound, NaiveDateTime, Utc};
use integration_s3::upload_file;
use reqwest::Client;
use tokio::signal::unix;

use integration_youtube::{
    data_api::videos::list_videos,
    thumbnail::get_thumbnail,
    youtubei::{
        live_chat::{LiveChatMessage, MemberMessageType, PaidMessageType},
        updated_metadata, updated_metadata_with_continuation, youtube_live_chat,
        youtube_live_chat_with_continuation,
    },
};
use vtstats_database::{
    channels::{get_channel_by_id, Channel},
    jobs::{queue_send_notification, CollectYoutubeStreamMetadataJobPayload},
    stream_events::{
        add_stream_events, StreamEventValue, YoutubeMemberMilestone, YoutubeNewMember,
        YoutubeSuperChat, YoutubeSuperSticker,
    },
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow, AddStreamViewerStatsQuery},
    streams::{
        delete_stream, end_stream, end_stream_with_values, get_stream_by_id, start_stream, Stream,
        StreamStatus,
    },
    PgPool,
};

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    client: Client,
    payload: CollectYoutubeStreamMetadataJobPayload,
) -> anyhow::Result<JobResult> {
    let Some(stream) = get_stream_by_id(payload.stream_id, pool).await? else {
        return Ok(JobResult::Completed);
    };

    let Some(channel) = get_channel_by_id(stream.channel_id, pool).await? else {
        return Ok(JobResult::Completed);
    };

    let (Ok(mut sigint), Ok(mut sigterm)) = (
        unix::signal(unix::SignalKind::interrupt()),
        unix::signal(unix::SignalKind::terminate()),
    ) else {
        anyhow::bail!("Failed to listen unix signal")
    };

    tokio::select! {
        _ = collect_stream_metadata(&stream, &client, pool) => {
            Ok(JobResult::Completed)
        },
        _ = collect_stream_chats(&channel, &stream, &client, pool) => {
            Ok(JobResult::Completed)
        },
        _ = sigint.recv() => {
            Ok(JobResult::Next { run: Utc::now() })
        },
        _ = sigterm.recv() => {
            Ok(JobResult::Next { run: Utc::now() })
        },
    }
}

async fn collect_stream_metadata(
    stream: &Stream,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut continuation: Option<String> = None;
    let mut status = stream.status;

    loop {
        let metadata = if let Some(continuation) = &continuation {
            updated_metadata_with_continuation(continuation, client).await
        } else {
            updated_metadata(&stream.platform_id, client).await
        }?;

        let (Some(timeout), Some(next_continuation)) =
            (metadata.timeout(), metadata.continuation())
        else {
            // stream not found
            if status == StreamStatus::Scheduled {
                delete_stream(stream.stream_id, pool).await?;
            } else {
                end_stream(stream.stream_id, pool).await?;
            }
            return Ok(());
        };

        continuation = Some(next_continuation.to_string());

        // stream is still waiting
        if metadata.is_waiting() {
            tokio::time::sleep(timeout).await;
            continue;
        }

        // record view stats
        if let Some(viewer) = metadata.view_count() {
            AddStreamViewerStatsQuery {
                time: Utc::now().duration_trunc(Duration::seconds(15)).unwrap(),
                count: viewer,
                stream_id: stream.stream_id,
            }
            .execute(pool)
            .await?;
        }

        // stream has ended
        if timeout.as_secs() > 5 {
            if status == StreamStatus::Scheduled {
                delete_stream(stream.stream_id, pool).await?;
                return Ok(());
            }

            let mut videos = list_videos(&stream.platform_id, client).await?;
            let video: Option<integration_youtube::data_api::videos::Stream> =
                videos.pop().and_then(Into::into);

            // update thumbnail url after stream is ended
            let thumbnail_url = match get_thumbnail(&stream.platform_id, client).await {
                Ok((filename, content_type, bytes)) => {
                    match upload_file(&filename, bytes, &content_type, client).await {
                        Ok(thumbnail_url) => Some(thumbnail_url),
                        Err(err) => {
                            tracing::error!(exception.stacktrace = ?err, message= %err);
                            None
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(exception.stacktrace = ?err, message= %err);
                    None
                }
            };

            end_stream_with_values(
                stream.stream_id,
                video.as_ref().map(|s| s.title.as_str()),
                video.as_ref().and_then(|s| s.schedule_time),
                video.as_ref().and_then(|s| s.start_time),
                video.as_ref().and_then(|s| s.end_time),
                video.as_ref().and_then(|s| s.likes),
                thumbnail_url,
                pool,
            )
            .await?;

            queue_send_notification(
                Utc::now(),
                "youtube".into(),
                stream.platform_id.to_string(),
                stream.vtuber_id.to_string(),
                pool,
            )
            .await?;

            return Ok(());
        }

        if status == StreamStatus::Scheduled {
            start_stream(
                stream.stream_id,
                metadata.title().as_deref(),
                Utc::now(),
                metadata.like_count(),
                pool,
            )
            .await?;
            status = StreamStatus::Live;
        }

        tokio::time::sleep(timeout).await;
    }
}

async fn collect_stream_chats(
    channel: &Channel,
    stream: &Stream,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    use std::time::Duration;

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
