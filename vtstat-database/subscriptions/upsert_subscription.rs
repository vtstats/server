use sqlx::{postgres::PgQueryResult, types::Json, PgPool, Result};

use super::TelegramSubscriptionPayload;

pub struct UpsertSubscriptionQuery {
    pub subscription_id: Option<i32>,
    pub payload: TelegramSubscriptionPayload,
}

impl UpsertSubscriptionQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        match self.subscription_id {
            Some(id) => {
                let query = sqlx::query!(
                    "UPDATE subscriptions \
                    SET payload = $1, updated_at = NOW() \
                    WHERE subscription_id = $2",
                    Json(self.payload) as _,
                    id,
                )
                .execute(pool);

                crate::otel::instrument("UPDATE", "subscriptions", query).await
            }
            None => {
                let query = sqlx::query!(
                    "INSERT INTO subscriptions (kind, payload) \
                    VALUES ('telegram_stream_update', $1)",
                    Json(self.payload) as _
                )
                .execute(pool);

                crate::otel::instrument("INSERT INTO", "subscriptions", query).await
            }
        }
    }
}
