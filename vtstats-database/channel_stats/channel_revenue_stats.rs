use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

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
    .try_map(|r| {
        let ts = r.ts.timestamp_millis();
        let map = serde_json::from_value(r.v1).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        Ok((ts, map))
    })
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
        let map = serde_json::from_value(r.value).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        Ok(
            ChannelRevenueStatsRow {
                channel_id: r.channel_id,
                value: map,
            }
        )
    })
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_revenue_stats", query)
}
