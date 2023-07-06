use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Result};

#[derive(Serialize, Deserialize)]
pub struct CreateVTuber {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub twitter_username: Option<String>,
    pub thumbnail_url: Option<String>,
}

impl CreateVTuber {
    pub async fn execute(self, pool: &PgPool) -> Result<()> {
        let query = sqlx::query!(
            "INSERT INTO vtubers (vtuber_id, native_name, english_name, japanese_name, twitter_username, thumbnail_url)
            VALUES ($1, $2, $3, $4, $5, $6)",
            self.vtuber_id,
            self.native_name,
            self.english_name,
            self.japanese_name,
            self.twitter_username,
            self.thumbnail_url
        )
        .execute(pool);

        crate::otel::instrument("INSERT", "vtubers", query).await?;

        Ok(())
    }
}
