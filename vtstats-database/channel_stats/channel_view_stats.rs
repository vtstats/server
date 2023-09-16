use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use crate::channel_stats::AddChannelViewStatsRow;

pub async fn channel_view_stats(
    channel_id: i32,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<(i64, i32)>> {
    let query = sqlx::query!(
        r#"
 SELECT time ts, count v1
   FROM channel_view_stats
  WHERE channel_id = $1
    AND (time >= $2 OR $2 IS NULL)
    AND (time <= $3 OR $3 IS NULL)
        "#,
        channel_id, // $1
        start_at,   // $2
        end_at,     // $3
    )
    .map(|row| (row.ts.timestamp_millis(), row.v1))
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_view_stats", query)
}

pub async fn channel_view_stats_at(
    at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<AddChannelViewStatsRow>> {
    let query = sqlx::query_as!(
        AddChannelViewStatsRow,
        "SELECT channel_id, count as value FROM channel_view_stats WHERE time = $1",
        at,
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_view_stats", query)
}

// TODO: add unit tests
