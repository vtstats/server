use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Result};

#[derive(Serialize)]
pub struct StreamViewerStats {
    pub time: DateTime<Utc>,
    pub count: i32,
}

pub async fn stream_viewer_stats(stream_id: i32, pool: &PgPool) -> Result<Vec<StreamViewerStats>> {
    let query = sqlx::query_as!(
        StreamViewerStats,
        r#"
 SELECT time, count
   FROM stream_viewer_stats
  WHERE stream_id = $1
        "#,
        stream_id,
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_viewer_stats", query).await
}

// TODO add unit tests
