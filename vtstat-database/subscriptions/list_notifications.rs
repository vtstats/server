use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, PgPool, Result, Row};

pub struct ListNotificationsQuery {
    pub subscription_id: i32,
}

pub struct Notification {
    pub notification_id: i32,
    pub payload: NotificationPayload,
}

#[derive(Deserialize, Serialize)]
pub struct NotificationPayload {
    pub vtuber_id: String,
    pub stream_id: i32,
    pub message_id: i64,
}

impl FromRow<'_, PgRow> for Notification {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Notification {
            notification_id: row.try_get("notification_id")?,
            payload: row.try_get::<Json<NotificationPayload>, _>("payload")?.0,
        })
    }
}

impl ListNotificationsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Option<Notification>> {
        let query = sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE subscription_id = $1",
        )
        .bind(self.subscription_id)
        .fetch_optional(pool);

        crate::otel::instrument("SELECT", "notifications", query).await
    }
}
