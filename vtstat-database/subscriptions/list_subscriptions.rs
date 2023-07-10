use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool, Result, Row};

pub enum ListTelegramSubscriptionQuery {
    ByVtuberId(String),
    ByChatId(i64),
}

#[derive(Debug)]
pub enum ListDiscordSubscriptionQuery {
    ByChannelId(String),
    ByVtuberId(String),
}

#[derive(Debug, Serialize)]
pub struct Subscription<Payload: DeserializeOwned + Debug> {
    pub subscription_id: i32,
    pub payload: Payload,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramSubscriptionPayload {
    pub vtuber_id: String,
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
            // kind: row.try_get("kind")?,
            payload: row.try_get::<Json<Payload>, _>("payload")?.0,
            updated_at: row.try_get("updated_at")?,
            created_at: row.try_get("created_at")?,
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
    pub async fn execute(self, pool: &PgPool) -> anyhow::Result<i32> {
        let x = sqlx::query!(
            "SELECT COUNT(*) FROM vtubers WHERE vtuber_id = $1",
            self.payload.vtuber_id
        )
        .fetch_one(pool)
        .await?;

        anyhow::ensure!(
            matches!(x.count, Some(c) if c > 0),
            "VTuber id {:?} didn't existed",
            &self.payload.vtuber_id
        );

        let row = sqlx::query(
            "INSERT INTO subscriptions (kind, payload) \
            VALUES ('discord_stream_update', $1) \
            RETURNING subscription_id",
        )
        .bind(&Json(&self.payload))
        .fetch_one(pool)
        .await?;

        Ok(row.try_get::<i32, _>("subscription_id")?)
    }
}

// TODO add unit tests

pub async fn list_subscriptions(
    pool: &PgPool,
) -> Result<Vec<Subscription<DiscordSubscriptionPayload>>> {
    sqlx::query_as::<_, Subscription<DiscordSubscriptionPayload>>("SELECT * FROM subscriptions")
        .fetch_all(pool)
        .await
}
