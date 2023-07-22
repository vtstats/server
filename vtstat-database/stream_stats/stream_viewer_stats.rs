use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct StreamViewerStatsQuery {
    platform: String,
    platform_stream_id: String,
}

#[derive(sqlx::FromRow)]
pub struct StreamViewerStats {
    pub time: DateTime<Utc>,
    pub count: i32,
}

impl StreamViewerStatsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<StreamViewerStats>> {
        let query = sqlx::query_as::<_, StreamViewerStats>(
            r#"
     SELECT time, count
       FROM stream_viewer_stats
      WHERE stream_id IN
            (
                SELECT stream_id
                  FROM streams
                 WHERE platform::TEXT = $1
                   AND platform_id = $2
            )
            "#,
        )
        .bind(self.platform)
        .bind(self.platform_stream_id)
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "stream_viewer_stats", query).await
    }
}

// TODO add unit tests
