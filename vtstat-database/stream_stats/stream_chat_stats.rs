use sqlx::{PgPool, Result};

use crate::SeriesData2;

pub async fn stream_chat_stats(stream_id: i32, pool: &PgPool) -> Result<Vec<SeriesData2>> {
    let query = sqlx::query_as!(
        SeriesData2,
        r#"
 SELECT time ts, count v1, from_member_count v2
   FROM stream_chat_stats
  WHERE stream_id = $1
        "#,
        stream_id,
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_chat_stats", query).await
}

// TODO: add unit tests
