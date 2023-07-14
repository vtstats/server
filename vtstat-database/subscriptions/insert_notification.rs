use sqlx::{postgres::PgQueryResult, types::Json, PgPool, Result};

use super::NotificationPayload;

pub struct InsertNotificationQuery {
    pub subscription_id: i32,

    pub payload: NotificationPayload,
}

impl InsertNotificationQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let query = sqlx::query(
            "INSERT INTO notifications (subscription_id, payload) \
            VALUES ($1, $2)",
        )
        .bind(self.subscription_id)
        .bind(Json(self.payload))
        .execute(pool);

        crate::otel::instrument("INSERT INTO", "notifications", query).await
    }
}