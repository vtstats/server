use sqlx::{PgPool, Result};

pub async fn stream_chat_stats(stream_id: i32, pool: &PgPool) -> Result<Vec<(i64, i32, i32)>> {
    let query = sqlx::query!(
        r#"
 SELECT time ts, count v1, from_member_count v2
   FROM stream_chat_stats
  WHERE stream_id = $1
        "#,
        stream_id,
    )
    .map(|row| (row.ts.timestamp_millis(), row.v1, row.v2))
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_chat_stats", query).await
}

// TODO: add unit tests
