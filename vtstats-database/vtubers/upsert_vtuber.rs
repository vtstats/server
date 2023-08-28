use sqlx::{PgExecutor, Result};

pub struct UpsertVTuber {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub twitter_username: Option<String>,
    pub thumbnail_url: Option<String>,
}

impl UpsertVTuber {
    pub async fn execute(self, executor: impl PgExecutor<'_>) -> Result<()> {
        let query = sqlx::query!(
            "INSERT INTO vtubers AS v (vtuber_id, native_name, english_name, japanese_name, twitter_username, thumbnail_url) \
            VALUES ($1, $2, $3, $4, $5, $6) \
            ON CONFLICT (vtuber_id) DO UPDATE \
            SET native_name = COALESCE($2, v.native_name), \
            english_name = COALESCE($3, v.english_name), \
            japanese_name = COALESCE($4, v.japanese_name), \
            twitter_username = COALESCE($5, v.twitter_username),\
            thumbnail_url = COALESCE($6, v.thumbnail_url)",
            self.vtuber_id,
            self.native_name,
            self.english_name,
            self.japanese_name,
            self.twitter_username,
            self.thumbnail_url
        )
        .execute(executor);

        crate::otel::instrument("INSERT", "vtubers", query).await?;

        Ok(())
    }
}
