use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use crate::{channel_stats::AddChannelViewStatsRow, SeriesData};

pub async fn channel_view_stats(
    channel_id: i32,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<SeriesData>> {
    let query = sqlx::query_as!(
        SeriesData,
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
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channel_view_stats", query).await
}

pub async fn channel_view_stats_before(
    before: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<AddChannelViewStatsRow>> {
    let query = sqlx::query_as!(
        AddChannelViewStatsRow,
        "SELECT channel_id, count FROM channel_view_stats WHERE (time, channel_id) IN \
        (SELECT MAX(time), channel_id FROM channel_view_stats WHERE time <= $1 GROUP BY channel_id)",
        before
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channel_view_stats", query).await
}

// TODO: add unit tests
