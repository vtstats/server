use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use crate::json::decode_json_value;

use super::ChannelRevenueStatsRow;

pub async fn channel_revenue_stats(
    channel_id: i32,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<(i64, HashMap<String, f32>)>> {
    let query = sqlx::query!(
        r#"
 SELECT time ts, value v1
   FROM channel_revenue_stats
  WHERE channel_id = $1
    AND (time >= $2 OR $2 IS NULL)
    AND (time <= $3 OR $3 IS NULL)
        "#,
        channel_id, // $1
        start_at,   // $2
        end_at,     // $3
    )
    .try_map(|r| Ok((r.ts.timestamp_millis(), decode_json_value(r.v1)?)))
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_revenue_stats", query)
}

pub async fn channel_revenue_stats_before(
    before: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenueStatsRow>> {
    let query = sqlx::query!(
        "SELECT channel_id, value FROM channel_revenue_stats WHERE (time, channel_id) IN \
        (SELECT MAX(time), channel_id FROM channel_revenue_stats WHERE time <= $1 GROUP BY channel_id)",
        before
    )
    .try_map(|r| {
        Ok(ChannelRevenueStatsRow {
            channel_id: r.channel_id,
            value: decode_json_value(r.value)?,
        })
    })
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_revenue_stats", query)
}
