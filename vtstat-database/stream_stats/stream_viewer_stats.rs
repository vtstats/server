use sqlx::{PgPool, Result};

pub async fn stream_viewer_stats(stream_id: i32, pool: &PgPool) -> Result<Vec<(i64, i32)>> {
    let query = sqlx::query!(
        r#"
 SELECT time ts, count v1
   FROM stream_viewer_stats
  WHERE stream_id = $1
        "#,
        stream_id,
    )
    .map(|row| (row.ts.timestamp_millis(), row.v1))
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_viewer_stats", query).await
}

// TODO add unit tests
