use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Write;
use vtstat_database::{
    subscriptions::{ListTelegramSubscriptionQuery, TelegramSubscriptionPayload, UpsertSubscriptionQuery},
    PgPool,
};
use warp::{Filter, Rejection, Reply};

use crate::filters::with_pool;

#[derive(Deserialize, Debug)]
pub struct Update {
    pub update_id: i64,
    pub message: Message,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub message_id: i64,
    pub text: String,
    pub chat: Chat,
}

#[derive(Deserialize, Debug)]
pub struct Chat {
    pub id: i64,
}

#[derive(Serialize, Debug)]
pub struct UpdateResponse {
    pub method: &'static str,
    pub parse_mode: &'static str,
    pub chat_id: i64,
    pub text: String,
}

async fn telegram_updates(
    pool: PgPool,
    update: Update,
    secret_token: String,
) -> Result<impl warp::Reply, Rejection> {
    if secret_token != env::var("TELEGRAM_SECRET_TOKEN").unwrap_or_default() {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }

    Ok(warp::reply::json(&UpdateResponse {
        method: "sendMessage",
        parse_mode: "HTML",
        chat_id: update.message.chat.id,
        text: execute_command(update.message.chat.id, update.message.text, &pool)
            .await
            .unwrap_or_else(|err| err.to_string()),
    })
    .into_response())
}

async fn execute_command(chat_id: i64, text: String, pool: &PgPool) -> anyhow::Result<String> {
    let mut words = text.split_ascii_whitespace().skip_while(|&x| x.is_empty());

    let subscription = ListTelegramSubscriptionQuery::ByChatId(chat_id)
        .execute(pool)
        .await?
        .into_iter()
        .next();

    match words.next() {
        Some("/info") => match subscription {
            Some(subscription) => {
                let mut buf = String::new();

                let _ = writeln!(buf, "Subscription");
                let _ = writeln!(buf, "<b>Chat ID: </b>{}", subscription.payload.chat_id);
                let _ = writeln!(
                    buf,
                    "<b>UTC Offset: </b>{}",
                    subscription
                        .payload
                        .utc_offset
                        .as_deref()
                        .unwrap_or("(Not set)")
                );
                let _ = writeln!(buf, "<b>VTubers: </b>");

                for (index, id) in subscription.payload.vtuber_ids.iter().enumerate() {
                    let _ = writeln!(buf, "{:>6}. {}", index + 1, id);
                }

                Ok(buf)
            }
            None => Ok(format!("Not subscription found.")),
        },

        Some("/timezone") => {
            let utc_offset = words
                .next()
                .ok_or_else(|| anyhow::anyhow!("Usage: /timezone timezone"))?
                .to_string();

            UpsertSubscriptionQuery {
                subscription_id: subscription.as_ref().map(|s| s.subscription_id),
                payload: TelegramSubscriptionPayload {
                    chat_id,
                    utc_offset: Some(utc_offset),
                    vtuber_ids: subscription
                        .map(|s| s.payload.vtuber_ids)
                        .unwrap_or_default(),
                },
            }
            .execute(&pool)
            .await?;

            Ok("Subscription updated".into())
        }

        Some("/add") => {
            let mut vtuber_ids = words
                .next()
                .unwrap_or_default()
                .split(',')
                .skip_while(|&x| x.is_empty())
                .map(|x| x.to_string())
                .collect();

            let vtuber_ids = subscription
                .as_ref()
                .map(|subscription| {
                    let mut ids = subscription.payload.vtuber_ids.clone();
                    ids.append(&mut vtuber_ids);
                    ids.dedup();
                    ids
                })
                .unwrap_or_else(|| vtuber_ids);

            UpsertSubscriptionQuery {
                subscription_id: subscription.as_ref().map(|s| s.subscription_id),
                payload: TelegramSubscriptionPayload {
                    chat_id,
                    utc_offset: subscription.and_then(|s| s.payload.utc_offset),
                    vtuber_ids,
                },
            }
            .execute(&pool)
            .await?;

            Ok("Subscription updated".into())
        }

        Some("/remove") => {
            let vtuber_ids: Vec<_> = words
                .next()
                .unwrap_or_default()
                .split(',')
                .skip_while(|&x| x.is_empty())
                .map(|x| x.to_string())
                .collect();

            let vtuber_ids = subscription
                .as_ref()
                .map(|subscription| {
                    subscription
                        .payload
                        .vtuber_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !vtuber_ids.contains(id))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vtuber_ids);

            UpsertSubscriptionQuery {
                subscription_id: subscription.as_ref().map(|s| s.subscription_id),
                payload: TelegramSubscriptionPayload {
                    chat_id,
                    utc_offset: subscription.and_then(|s| s.payload.utc_offset),
                    vtuber_ids,
                },
            }
            .execute(&pool)
            .await?;

            Ok("Subscription updated".into())
        }

        Some("/clear") => {
            UpsertSubscriptionQuery {
                subscription_id: subscription.as_ref().map(|s| s.subscription_id),
                payload: TelegramSubscriptionPayload {
                    chat_id,
                    utc_offset: subscription.and_then(|s| s.payload.utc_offset),
                    vtuber_ids: Vec::new(),
                },
            }
            .execute(&pool)
            .await?;

            Ok("Subscription updated.".into())
        }

        _ => anyhow::bail!("Invalid input"),
    }
}

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("telegram")
        .and(warp::post())
        .and(with_pool(pool))
        .and(warp::body::json())
        .and(warp::header::<String>("x-telegram-bot-api-secret-token"))
        .and_then(telegram_updates)
}
