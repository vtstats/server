use sqlx::{PgPool, Result};

use crate::SeriesData;

pub async fn stream_viewer_stats(stream_id: i32, pool: &PgPool) -> Result<Vec<SeriesData>> {
    let query = sqlx::query_as!(
        SeriesData,
        r#"
 SELECT time ts, count v1
   FROM stream_viewer_stats
  WHERE stream_id = $1
        "#,
        stream_id,
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_viewer_stats", query).await
}

// TODO add unit tests
