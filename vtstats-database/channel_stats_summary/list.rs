use sqlx::{PgPool, Result};

use super::{ChannelStatsKind, ChannelStatsSummary};

pub async fn list(
    channel_ids: &[i32],
    kind: ChannelStatsKind,
    pool: &PgPool,
) -> Result<Vec<ChannelStatsSummary>> {
    let query = sqlx::query_as!(
        ChannelStatsSummary,
        "SELECT channel_id, kind as \"kind: _\", updated_at, \
        value, value_1_day_ago, value_7_days_ago, value_30_days_ago \
        FROM channel_stats_summary \
        WHERE channel_id = ANY($1) AND kind = $2",
        channel_ids,
        kind as _
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_stats_summary", query)
}
