use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool, Result, Row};

pub struct ListNotificationsQuery {
    pub subscription_id: i32,
    pub stream_id: i32,
}

#[derive(Serialize)]
pub struct Notification {
    pub notification_id: i32,
    pub payload: NotificationPayload,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize)]
pub struct NotificationPayload {
    pub vtuber_id: String,
    pub stream_id: i32,
    pub message_id: String,
}

impl FromRow<'_, PgRow> for Notification {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Notification {
            notification_id: row.try_get("notification_id")?,
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

        crate::otel::instrument("SELECT", "notifications", query).await
    }
}

pub async fn list(pool: &PgPool) -> Result<Vec<Notification>> {
    let query = sqlx::query_as::<_, Notification>("SELECT * FROM notifications").fetch_all(pool);

    crate::otel::instrument("SELECT", "notifications", query).await
}
