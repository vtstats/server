use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct ListVtubersQuery;

pub struct VTuber {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub twitter_username: Option<String>,
    pub debuted_at: Option<DateTime<Utc>>,
    pub retired_at: Option<DateTime<Utc>>,
}

impl ListVtubersQuery {
    pub async fn execute(pool: &PgPool) -> Result<Vec<VTuber>> {
        let query = sqlx::query_as!(VTuber, "SELECT * FROM vtubers").fetch_all(pool);

        crate::otel::instrument("SELECT", "vtubers", query).await
    }
}
