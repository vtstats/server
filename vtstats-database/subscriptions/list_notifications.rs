use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool, Result, Row};

pub struct ListNotificationsQuery {
    pub subscription_id: i32,
    pub stream_id: i32,
}

#[derive(Serialize)]
pub struct Notification {
    pub notification_id: i32,
    pub subscription_id: i32,
    pub payload: NotificationPayload,
    #[serde(with = "ts_milliseconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_milliseconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize)]
pub struct NotificationPayload {
    pub vtuber_id: String,
    pub stream_id: i32,
    pub message_id: String,
    #[serde(default)]
    pub start_message_id: Option<String>,
    #[serde(default)]
    pub end_message_id: Option<String>,
}

impl FromRow<'_, PgRow> for Notification {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Notification {
            notification_id: row.try_get("notification_id")?,
            subscription_id: row.try_get("subscription_id")?,
            payload: row.try_get::<Json<_>, _>("payload")?.0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl ListNotificationsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Option<Notification>> {
        let query = sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications \
            WHERE subscription_id = $1 \
            AND (payload ->> 'stream_id')::int = $2",
        )
        .bind(self.subscription_id)
        .bind(self.stream_id)
        .fetch_optional(pool);

        crate::otel::execute_query!("SELECT", "notifications", query)
    }
}

pub async fn list(end_at: Option<DateTime<Utc>>, pool: &PgPool) -> Result<Vec<Notification>> {
    let query = sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications \
        WHERE (updated_at < $1 OR $1 is null) \
        ORDER BY updated_at DESC \
        LIMIT 24",
    )
    .bind(end_at)
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "notifications", query)
}
