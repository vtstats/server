use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct ListChannelsViewStats {
    pub platform: String,
    pub platform_channel_id: String,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
pub struct ChannelsViewStats {
    pub time: DateTime<Utc>,
    pub count: i32,
}

impl ListChannelsViewStats {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<ChannelsViewStats>> {
        let query = sqlx::query_as!(
            ChannelsViewStats,
            r#"
     SELECT time, count
       FROM channel_view_stats
      WHERE channel_id IN
            (
                SELECT channel_id
                  FROM channels
                 WHERE platform::TEXT = $1
                   AND platform_id = $2
            )
        AND (time >= $3 OR $3 IS NULL)
        AND (time <= $4 OR $4 IS NULL)
            "#,
            self.platform,            // $1
            self.platform_channel_id, // $2
            self.start_at,            // $3
            self.end_at,              // $4
        )
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "channel_view_stats", query).await
    }
}

// TODO: add unit tests
