use sqlx::{postgres::PgQueryResult, PgPool, Result};

pub struct UpdateNotificationQuery {
    pub notification_id: i32,
}

impl UpdateNotificationQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let query = sqlx::query!(
            "UPDATE notifications \
            SET updated_at = NOW() \
            WHERE notification_id = $1",
            self.notification_id
        )
        .execute(pool);

        crate::otel::execute_query!("UPDATE", "notifications", query)
    }
}
