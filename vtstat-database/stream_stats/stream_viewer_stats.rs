use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct StreamViewerStatsQuery {
    platform: String,
    platform_stream_id: String,
}

pub struct StreamViewerStats {
    pub time: DateTime<Utc>,
    pub count: i32,
}

impl StreamViewerStatsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<StreamViewerStats>> {
        sqlx::query_as!(
            StreamViewerStats,
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
            self.platform,
            self.platform_stream_id
        )
        .fetch_all(pool)
        .await
    }
}

// TODO add unit tests
