use sqlx::{PgExecutor, Result};

use super::Platform;

pub struct CreateChannel {
    pub platform: Platform,
    pub platform_id: String,
    pub vtuber_id: String,
}

impl CreateChannel {
    pub async fn execute(&self, executor: impl PgExecutor<'_>) -> Result<i32> {
        let query = sqlx::query!(
            "INSERT INTO channels (platform, platform_id, vtuber_id, kind) \
            VALUES($1, $2, $3, '') \
            RETURNING channel_id",
            self.platform as _,
            self.platform_id,
            self.vtuber_id,
        )
        .fetch_one(executor);

        let record = crate::otel::instrument("INSERT", "channels", query).await?;

        Ok(record.channel_id)
    }
}
