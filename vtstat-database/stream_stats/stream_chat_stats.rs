use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct StreamChatStatsQuery {
    pub platform: String,
    pub platform_stream_id: String,
}

pub struct StreamChatStats {
    pub time: DateTime<Utc>,
    pub count: i32,
    pub from_member_count: i32,
}

impl StreamChatStatsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<StreamChatStats>> {
        sqlx::query_as!(
            StreamChatStats,
            r#"
     SELECT time, count, from_member_count
       FROM stream_chat_stats
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

// TODO: add unit tests
