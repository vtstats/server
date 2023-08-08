use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::ChannelRevenueStatsRow;

pub async fn channel_revenue_stats_before(
    before: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenueStatsRow>> {
    let query = sqlx::query!(
        "SELECT channel_id, value FROM channel_revenue_stats WHERE (time, channel_id) IN \
        (SELECT MAX(time), channel_id FROM channel_revenue_stats WHERE time <= $1 GROUP BY channel_id)",
        before
    )
    .fetch_all(pool);

    let rows = crate::otel::instrument("SELECT", "channel_revenue_stats", query).await?;

    let rows: Vec<_> = rows
        .into_iter()
        .filter_map(|row| {
            Some(ChannelRevenueStatsRow {
                channel_id: row.channel_id,
                value: serde_json::from_value(row.value).ok()?,
            })
        })
        .collect();

    Ok(rows)
}
