use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, Result};

pub struct UpsertVTuber {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub twitter_username: Option<String>,
    pub thumbnail_url: Option<String>,
    pub retired_at: Option<DateTime<Utc>>,
}

impl UpsertVTuber {
    pub async fn execute(self, executor: impl PgExecutor<'_>) -> Result<()> {
        let query = sqlx::query!(
            "INSERT INTO vtubers AS v (vtuber_id, native_name, english_name, japanese_name, twitter_username, thumbnail_url, retired_at) \
            VALUES ($1, $2, $3, $4, $5, $6, $7) \
            ON CONFLICT (vtuber_id) DO UPDATE \
            SET native_name = $2, \
            english_name = $3, \
            japanese_name = $4, \
            twitter_username = $5, \
            thumbnail_url = COALESCE($6, v.thumbnail_url), \
            retired_at = $7",
            self.vtuber_id,
            self.native_name,
            self.english_name,
            self.japanese_name,
            self.twitter_username,
            self.thumbnail_url,
            self.retired_at,
        )
        .execute(executor);

        crate::otel::execute_query!("INSERT", "vtubers", query)?;

        Ok(())
    }
}
