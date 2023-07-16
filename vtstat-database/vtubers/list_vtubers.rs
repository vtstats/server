use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Result};

pub struct ListVtubersQuery;

#[derive(sqlx::FromRow, Serialize)]
pub struct VTuber {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub thumbnail_url: Option<String>,
    pub twitter_username: Option<String>,
    pub debuted_at: Option<DateTime<Utc>>,
    pub retired_at: Option<DateTime<Utc>>,
}

impl ListVtubersQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<VTuber>> {
        let query = sqlx::query_as::<_, VTuber>("SELECT * FROM vtubers").fetch_all(pool);

        crate::otel::instrument("SELECT", "vtubers", query).await
    }
}
