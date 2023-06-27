use std::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool, Result, Row};

pub enum ListTelegramSubscriptionQuery {
    ByVtuberId(String),
    ByChatId(i64),
}

pub enum ListDiscordSubscriptionQuery {
    ByChannelId(String),
    ByVtuberId(String),
}

#[derive(Debug)]
pub struct Subscription<Payload: DeserializeOwned + Debug> {
    pub subscription_id: i32,
    pub payload: Payload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramSubscriptionPayload {
    pub vtuber_ids: Vec<String>,
    pub utc_offset: Option<String>,
    pub chat_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DiscordSubscriptionPayload {
    pub vtuber_id: String,
    pub channel_id: String,
}

impl<Payload: DeserializeOwned + Debug> FromRow<'_, PgRow> for Subscription<Payload> {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Subscription {
            subscription_id: row.try_get("subscription_id")?,
            payload: row.try_get::<Json<Payload>, _>("payload")?.0,
        })
    }
}

impl ListTelegramSubscriptionQuery {
    pub async fn execute(
        self,
        pool: &PgPool,
    ) -> Result<Vec<Subscription<TelegramSubscriptionPayload>>> {
        let query = match self {
            ListTelegramSubscriptionQuery::ByVtuberId(id) => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'telegram_stream_update' \
                AND payload -> 'vtuber_ids' ? $1",
            )
            .bind(id),
            ListTelegramSubscriptionQuery::ByChatId(id) => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'telegram_stream_update' \
                AND (payload ->> 'chat_id')::int = $1",
            )
            .bind(id),
        }
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "subscriptions", query).await
    }
}

impl ListDiscordSubscriptionQuery {
    pub async fn execute(
        self,
        pool: &PgPool,
    ) -> Result<Vec<Subscription<DiscordSubscriptionPayload>>> {
        let query = match self {
            ListDiscordSubscriptionQuery::ByChannelId(channel_id) => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'discord_stream_update' \
                AND (payload ->> 'channel_id') = $1",
            )
            .bind(channel_id),
            ListDiscordSubscriptionQuery::ByVtuberId(vtuber_id) => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'discord_stream_update' \
                AND (payload ->> 'vtuber_id') = $1",
            )
            .bind(vtuber_id),
        }
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "subscriptions", query).await
    }
}

pub struct RemoveDiscordSubscriptionQuery {
    pub channel_id: String,
    pub subscription_id: i32,
}

impl RemoveDiscordSubscriptionQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "DELETE FROM subscriptions \
            WHERE kind = 'discord_stream_update' \
            AND (payload ->> 'channel_id') = $1 \
            AND subscription_id = $2",
            self.channel_id,
            self.subscription_id
        )
        .execute(pool)
        .await
        .map(|_| ())
    }
}

pub struct CreateDiscordSubscriptionQuery {
    pub payload: DiscordSubscriptionPayload,
}

impl CreateDiscordSubscriptionQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<i32> {
        sqlx::query(
            "INSERT INTO subscriptions (kind, payload) \
            VALUES ('discord_stream_update', $1) \
            RETURNING subscription_id",
        )
        .bind(&Json(&self.payload))
        .fetch_one(pool)
        .await
        .and_then(|r| r.try_get::<i32, _>("subscription_id"))
    }
}

// TODO add unit tests
