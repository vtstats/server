use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct ListChannelsSubscriberStats {
    pub platform: String,
    pub platform_channel_id: String,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
pub struct ChannelsSubscriberStats {
    pub time: DateTime<Utc>,
    pub count: i32,
}

impl ListChannelsSubscriberStats {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<ChannelsSubscriberStats>> {
        let query = sqlx::query_as::<_, ChannelsSubscriberStats>(
            r#"
     SELECT time, count
       FROM channel_subscriber_stats
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
        )
        .bind(self.platform) // $1
        .bind(self.platform_channel_id) // $2
        .bind(self.start_at) // $3
        .bind(self.end_at) // $4
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "channel_subscriber_stats", query).await
    }
}

// TODO: add unit tests
