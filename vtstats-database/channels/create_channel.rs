use sqlx::{PgExecutor, Result};

use super::Platform;

pub struct CreateChannel {
    pub platform: Platform,
    pub platform_id: String,
    pub vtuber_id: String,
    pub kind: Option<String>,
}

impl CreateChannel {
    pub async fn execute(&self, executor: impl PgExecutor<'_>) -> Result<i32> {
        let query = sqlx::query!(
            "INSERT INTO channels (platform, platform_id, vtuber_id, kind) \
            VALUES($1, $2, $3, $4) \
            RETURNING channel_id",
            self.platform as _,
            self.platform_id,
            self.vtuber_id,
            self.kind.as_deref().unwrap_or_default(),
        )
        .fetch_one(executor);

        let record = crate::otel::execute_query!("INSERT", "channels", query)?;

        Ok(record.channel_id)
    }
}
