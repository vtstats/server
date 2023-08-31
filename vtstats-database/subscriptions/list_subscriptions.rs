use std::fmt::Debug;

use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool, Result, Row};

use crate::json::decode_json_value;

use super::NotificationPayload;

pub enum ListTelegramSubscriptionQuery {
    ByVtuberId(String),
    ByChatId(i64),
}

#[derive(Debug)]
pub enum ListDiscordSubscriptionQuery {
    ByGuildId {
        guild_id: String,
    },
    ByChannelId {
        guild_id: String,
        channel_id: String,
    },
    ByVtuberId(String),
}

#[derive(Debug, Serialize)]
pub struct Subscription<Payload: DeserializeOwned + Debug> {
    pub subscription_id: i32,
    pub payload: Payload,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
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
    pub guild_id: String,
    pub vtuber_id: String,
    pub channel_id: String,
}

impl<Payload: DeserializeOwned + Debug> FromRow<'_, PgRow> for Subscription<Payload> {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Subscription {
            subscription_id: row.try_get("subscription_id")?,
            // kind: row.try_get("kind")?,
            payload: row.try_get::<Json<_>, _>("payload")?.0,
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

        crate::otel::execute_query!("SELECT", "subscriptions", query)
    }
}

impl ListDiscordSubscriptionQuery {
    pub async fn execute(
        self,
        pool: &PgPool,
    ) -> Result<Vec<Subscription<DiscordSubscriptionPayload>>> {
        let query = match self {
            ListDiscordSubscriptionQuery::ByGuildId { guild_id } => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'discord_stream_update' \
                AND (payload ->> 'guild_id') = $1",
            )
            .bind(guild_id),
            ListDiscordSubscriptionQuery::ByChannelId {
                guild_id,
                channel_id,
            } => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'discord_stream_update' \
                AND (payload ->> 'guild_id') = $1 \
                AND (payload ->> 'channel_id') = $2",
            )
            .bind(guild_id)
            .bind(channel_id),
            ListDiscordSubscriptionQuery::ByVtuberId(vtuber_id) => sqlx::query_as(
                "SELECT * FROM subscriptions \
                WHERE kind = 'discord_stream_update' \
                AND (payload ->> 'vtuber_id') = $1",
            )
            .bind(vtuber_id),
        }
        .fetch_all(pool);

        crate::otel::execute_query!("SELECT", "subscriptions", query)
    }
}

pub struct DiscordSubscriptionAndNotification {
    pub subscription_id: i32,
    pub subscription_payload: DiscordSubscriptionPayload,
    pub notification_id: Option<i32>,
    pub notification_payload: Option<NotificationPayload>,
}

pub async fn list_discord_subscription_and_notification_by_vtuber_id(
    vtuber_id: String,
    stream_id: i32,
    pool: &PgPool,
) -> Result<Vec<DiscordSubscriptionAndNotification>> {
    let query = sqlx::query!(
        "SELECT s.subscription_id id1, s.payload p1, n.payload as \"p2?\", n.notification_id as \"id2?\" \
        FROM subscriptions s \
        LEFT JOIN notifications n \
        ON s.subscription_id = n.subscription_id \
        AND (n.payload->>'stream_id')::int = $1 \
        WHERE s.kind = 'discord_stream_update' \
        AND (s.payload->>'vtuber_id') = $2",
        stream_id,
        vtuber_id,
    )
    .try_map(|r| {
        Ok(DiscordSubscriptionAndNotification {
            subscription_id: r.id1,
            subscription_payload: decode_json_value(r.p1)?,
            notification_id: r.id2,
            notification_payload: r.p2.map(decode_json_value).transpose()?,
        })
    })
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "subscriptions", query)
}

pub struct RemoveDiscordSubscriptionQuery {
    pub guild_id: String,
    pub channel_id: String,
    pub vtuber_id: String,
}

impl RemoveDiscordSubscriptionQuery {
    pub async fn execute(self, pool: &PgPool) -> anyhow::Result<()> {
        let query = sqlx::query!(
            "SELECT subscription_id FROM subscriptions \
            WHERE kind = 'discord_stream_update' \
            AND (payload ->> 'channel_id') = $1 \
            AND (payload ->> 'guild_id') = $2 \
            AND (payload ->> 'vtuber_id') = $3",
            self.channel_id,
            self.guild_id,
            self.vtuber_id
        )
        .fetch_optional(pool);

        let row = crate::otel::execute_query!("SELECT", "subscriptions", query)?;

        let Some(subscription_id) = row.map(|r| r.subscription_id) else {
            anyhow::bail!(
                "cannot found subscription `{}` in this channel.",
                self.vtuber_id
            )
        };

        let mut tx = pool.begin().await?;

        // notifications table contains reference to subscriptions table
        // so we need to remove these first
        sqlx::query!(
            "DELETE FROM notifications WHERE subscription_id = $1",
            subscription_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "DELETE FROM subscriptions WHERE subscription_id = $1",
            subscription_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

pub struct CreateDiscordSubscriptionQuery {
    pub payload: DiscordSubscriptionPayload,
}

impl CreateDiscordSubscriptionQuery {
    pub async fn execute(self, pool: &PgPool) -> anyhow::Result<i32> {
        let query = sqlx::query!(
            "SELECT COUNT(*) FROM vtubers WHERE vtuber_id = $1",
            self.payload.vtuber_id
        )
        .fetch_one(pool);

        let row = crate::otel::execute_query!("SELECT", "vtubers", query)?;

        anyhow::ensure!(
            matches!(row.count, Some(c) if c > 0),
            "VTuber id `{}` does not exist.",
            self.payload.vtuber_id
        );

        let query = sqlx::query!(
            "INSERT INTO subscriptions (kind, payload) \
            VALUES ('discord_stream_update', $1) \
            ON CONFLICT DO NOTHING \
            RETURNING subscription_id",
            Json(&self.payload) as _
        )
        .fetch_optional(pool);

        let record = crate::otel::execute_query!("INSERT", "subscriptions", query)?;

        let Some(record) = record else {
            anyhow::bail!("subscription `{}` already exists.", self.payload.vtuber_id)
        };

        Ok(record.subscription_id)
    }
}

// TODO add unit tests

pub async fn list_subscriptions(
    pool: &PgPool,
) -> Result<Vec<Subscription<DiscordSubscriptionPayload>>> {
    let query = sqlx::query_as::<_, Subscription<DiscordSubscriptionPayload>>(
        "SELECT * FROM subscriptions",
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "subscriptions", query)
}
