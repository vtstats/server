use chrono::{DateTime, Duration, DurationRound, NaiveDateTime, Utc};
use vtstat_database::{
    currencies::{Currency, ListCurrenciesQuery},
    donations::{
        AddDonationQuery, AddDonationRow, DonationValue, YoutubeMemberMilestoneDonationValue,
        YoutubeNewMemberDonationValue, YoutubeSuperChatDonationValue,
        YoutubeSuperStickerDonationValue,
    },
    jobs::CollectYoutubeStreamMetadataJobPayload,
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow, AddStreamViewerStatsQuery},
    streams::{EndStreamQuery, StartStreamQuery},
    PgPool,
};
use vtstat_request::{
    response::{MemberMessageType, PaidMessageType},
    LiveChatMessage, RequestHub,
};

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

    let (metadata_continuation, chat_continuation) = continuation
        .and_then(|c| serde_json::from_str::<(String, String)>(&c).ok())
        .unwrap_or_default();

    let metadata = if !metadata_continuation.is_empty() {
        hub.updated_metadata_with_continuation(&metadata_continuation)
            .await
    } else {
        hub.updated_metadata(&platform_stream_id).await
    }?;

    match (
        metadata.timeout(),
        metadata.continuation(),
        metadata.is_waiting(),
    ) {
        // stream not found
        (None, _, _) | (_, None, _) => {
            EndStreamQuery {
                stream_id,
                end_time: Some(Utc::now()),
                ..Default::default()
            }
            .execute(pool)
            .await?;

            Ok(JobResult::Completed {})
        }

        // stream is still waiting
        (Some(timeout), Some(continuation), true) => Ok(JobResult::Next {
            run: Utc::now() + Duration::from_std(timeout)?,
            continuation: Some(continuation.to_string()),
        }),

        // stream is still going or it has ended
        (Some(mut timeout), Some(next_metadata_continuation), false) => {
            let (messages, next_chat_timeout, next_chat_continuation) =
                if !chat_continuation.is_empty() {
                    hub.youtube_live_chat_with_continuation(chat_continuation)
                        .await
                } else {
                    hub.youtube_live_chat(&platform_channel_id, &platform_stream_id)
                        .await
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
                let stream = hub
                    .youtube_streams(&[platform_stream_id.to_string()])
                    .await?
                    .pop();

                EndStreamQuery {
                    stream_id,
                    updated_at: Some(now),
                    title: stream.as_ref().map(|s| s.title.as_str()),
                    end_time: stream.as_ref().and_then(|s| s.end_time),
                    start_time: stream.as_ref().and_then(|s| s.start_time),
                    schedule_time: stream.as_ref().and_then(|s| s.schedule_time),
                    likes: stream.as_ref().and_then(|s| s.likes),
                }
                .execute(pool)
                .await?;

                return Ok(JobResult::Completed);
            }

            StartStreamQuery {
                stream_id,
                title: metadata.title().as_deref(),
                start_time: now,
                likes: metadata.like_count(),
            }
            .execute(pool)
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
    }
}

async fn collect_donation_and_chat(
    stream_id: i32,
    messages: Vec<LiveChatMessage>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }

    let mut donation_rows = Vec::<AddDonationRow>::new();

    let mut chat_stats_rows = Vec::<AddStreamChatStatsRow>::new();

    let currencies = ListCurrenciesQuery.execute(pool).await?;

    for message in messages {
        match message {
            LiveChatMessage::Text {
                timestamp, badges, ..
            } => {
                let Some (time)  = parse_timestamp(&timestamp) else {
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
                    continue;
                };

                let value = match ty {
                    MemberMessageType::New => {
                        DonationValue::YoutubeNewMember(YoutubeNewMemberDonationValue {
                            message: text,
                            author_name,
                            author_badges: badges.join(","),
                            author_channel_id,
                        })
                    }
                    MemberMessageType::Milestone => {
                        DonationValue::YoutubeMemberMilestone(YoutubeMemberMilestoneDonationValue {
                            author_name,
                            author_badges: badges.join(","),
                            author_channel_id,
                        })
                    }
                };

                donation_rows.push(AddDonationRow { time, value })
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
                    continue;
                };

                let Some((paid_currency_code, paid_amount)) = parse_amount(&amount, &currencies) else {
                    continue;
                };

                let value = match ty {
                    PaidMessageType::SuperChat => {
                        DonationValue::YoutubeSuperChat(YoutubeSuperChatDonationValue {
                            paid_amount,
                            paid_currency_code: paid_currency_code.into(),
                            paid_color: color,
                            message: text,
                            author_name,
                            author_badges: badges.join(","),
                            author_channel_id,
                        })
                    }
                    PaidMessageType::SuperSticker => {
                        DonationValue::YoutubeSuperSticker(YoutubeSuperStickerDonationValue {
                            paid_amount,
                            paid_currency_code: paid_currency_code.into(),
                            paid_color: color,
                            message: text,
                            author_name,
                            author_badges: badges.join(","),
                            author_channel_id,
                        })
                    }
                };

                donation_rows.push(AddDonationRow { time, value })
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
        AddDonationQuery {
            stream_id,
            rows: donation_rows,
        }
        .execute(pool)
        .await?;
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

fn parse_amount<'a>(amount: &str, currencies: &'a [Currency]) -> Option<(&'a str, String)> {
    let i = amount.find(|ch: char| ch.is_ascii_digit())?;

    let (mut symbol, mut value) = amount.split_at(i);

    symbol = symbol.trim();
    value = value.trim();

    if symbol.is_empty() || value.is_empty() {
        return None;
    }

    let currency = currencies
        .iter()
        .find(|c| c.symbol == symbol || c.code == symbol)?;

    let value: f32 = value.replace(',', "").parse().ok()?;

    Some((&currency.code, format!("{value:.2}")))
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
    let currencies = vec![
        Currency {
            symbol: "$".into(),
            code: "USD".into(),
        },
        Currency {
            symbol: "¥".into(),
            code: "JPY".into(),
        },
    ];

    assert_eq!(parse_amount("", &currencies), None);
    assert_eq!(parse_amount("$", &currencies), None);
    assert_eq!(parse_amount("¥¥", &currencies), None);
    assert_eq!(parse_amount("JPY", &currencies), None);
    assert_eq!(parse_amount("10.00", &currencies), None);
    assert_eq!(parse_amount("1.0.00", &currencies), None);
    assert_eq!(parse_amount("$10.0.00", &currencies), None);
    assert_eq!(
        parse_amount("$1,000.00", &currencies),
        Some(("USD", "1000.00".into()))
    );
    assert_eq!(
        parse_amount("JPY 99.99", &currencies),
        Some(("JPY", "99.99".into()))
    );
    assert_eq!(
        parse_amount("¥99.9999", &currencies),
        Some(("JPY", "100.00".into()))
    );
    assert_eq!(
        parse_amount("USD 99.9901", &currencies),
        Some(("USD", "99.99".into()))
    );
}
