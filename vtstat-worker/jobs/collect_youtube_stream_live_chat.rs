use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use vtstat_database::{
    currencies::{Currency, ListCurrenciesQuery},
    donations::{
        AddDonationQuery, AddDonationRow, DonationValue, YoutubeMemberMilestoneDonationValue,
        YoutubeNewMemberDonationValue, YoutubeSuperChatDonationValue,
        YoutubeSuperStickerDonationValue,
    },
    jobs::CollectYoutubeStreamLiveChatJobPayload,
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow},
    PgPool,
};
use vtstat_request::{
    chat::response::{MemberMessageType, PaidMessageType},
    LiveChatMessage, RequestHub,
};

use super::JobResult;
use crate::timer::trunc_secs;

pub async fn execute(
    pool: &PgPool,
    hub: RequestHub,
    continuation: Option<String>,
    payload: CollectYoutubeStreamLiveChatJobPayload,
) -> anyhow::Result<JobResult> {
    let currencies = ListCurrenciesQuery.execute(pool).await?;

    let (messages, continuation) = match continuation {
        Some(continuation) => hub.youtube_live_chat_with_continuation(continuation).await,
        None => {
            hub.youtube_live_chat(&payload.platform_channel_id, &payload.platform_stream_id)
                .await
        }
    }?;

    let mut donation_rows = Vec::<AddDonationRow>::new();

    let mut chat_stats_rows = Vec::<AddStreamChatStatsRow>::new();

    for message in messages {
        match message {
            LiveChatMessage::Text {
                timestamp, badges, ..
            } => {
                let Some (time)  = parse_timestamp(&timestamp) else {
                    continue;
                };

                let time = trunc_secs(time, 15);

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
            stream_id: payload.stream_id,
            rows: chat_stats_rows,
        }
        .execute(pool)
        .await?;
    }

    if !donation_rows.is_empty() {
        AddDonationQuery {
            stream_id: payload.stream_id,
            rows: donation_rows,
        }
        .execute(pool)
        .await?;
    }

    match continuation.and_then(|c| c.get_continuation_and_timeout()) {
        Some((timeout, cont)) => Ok(JobResult::Next {
            run: Utc::now() + Duration::from_std(timeout)?,
            continuation: Some(cont),
        }),
        _ => Ok(JobResult::Completed),
    }
}

fn parse_timestamp(string: &str) -> Option<DateTime<Utc>> {
    if string.len() <= 6 {
        return None;
    }

    let (secs, nsecs) = string.split_at(string.len() - 6);

    Some(DateTime::from_utc(
        NaiveDateTime::from_timestamp(secs.parse().ok()?, nsecs.parse().ok()?),
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
            NaiveDateTime::from_timestamp(1620129525, 0),
            Utc,
        ))
    );
    assert_eq!(
        parse_timestamp("1620129525606474"),
        Some(DateTime::from_utc(
            NaiveDateTime::from_timestamp(1620129525, 606474),
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
