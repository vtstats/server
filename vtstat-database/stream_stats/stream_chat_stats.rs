use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Result};

#[derive(Serialize)]
pub struct StreamChatStats {
    pub time: DateTime<Utc>,
    pub count: i32,
    pub from_member_count: i32,
}

pub async fn stream_chat_stats(stream_id: i32, pool: &PgPool) -> Result<Vec<StreamChatStats>> {
    let query = sqlx::query_as!(
        StreamChatStats,
        r#"
 SELECT time, count, from_member_count
   FROM stream_chat_stats
  WHERE stream_id = $1
        "#,
        stream_id,
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_chat_stats", query).await
}

// TODO: add unit tests
