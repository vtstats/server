use sqlx::{PgPool, Result, Row};

use super::Platform;

pub struct CreateChannel {
    pub platform: Platform,
    pub platform_id: String,
    pub vtuber_id: String,
}

impl CreateChannel {
    pub async fn execute(&self, pool: &PgPool) -> Result<i32> {
        let query = sqlx::query(
            "INSERT INTO channels (platform, platform_id, vtuber_id, kind) \
            VALUES($1, $2, $3, '') \
            RETURNING channel_id",
        )
        .bind(&self.platform)
        .bind(&self.platform_id)
        .bind(&self.vtuber_id)
        .fetch_one(pool);

        crate::otel::instrument("INSERT", "channels", query)
            .await
            .and_then(|row| row.try_get("channel_id"))
    }
}
