use serde_json::json;
use sqlx::{PgExecutor, Result};

use super::ChannelStatsKind;

pub async fn create(
    channel_id: i32,
    kind: ChannelStatsKind,
    pool: impl PgExecutor<'_>,
) -> Result<()> {
    let value = match kind {
        ChannelStatsKind::View | ChannelStatsKind::Subscriber => json!(0),
        ChannelStatsKind::Revenue => json!({}),
    };

    let query = sqlx::query!(
        "INSERT INTO channel_stats_summary \
        (channel_id, kind, value, value_1_day_ago, value_7_days_ago, value_30_days_ago) \
        VALUES ($1, $2, $3, $3, $3, $3) \
        ON CONFLICT (channel_id, kind) DO NOTHING",
        channel_id, // $1
        kind as _,  // $2
        value,      // $3
    )
    .execute(pool);
    crate::otel::execute_query!("INSERT", "channel_stats_summary", query)?;

    Ok(())
}
