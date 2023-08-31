use sqlx::{PgPool, Result};

use crate::channels::ChannelWithStats;

use super::Channel;

/// list all bilibili channels but excluding retired
pub async fn list_youtube_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE platform = 'youtube' AND vtuber_id IN \
        (SELECT vtuber_id FROM vtubers WHERE retired_at IS NULL OR retired_at + '2 week' >= NOW())"
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channels", query)
}

/// list all bilibili channels but excluding retired
pub async fn list_bilibili_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE platform = 'bilibili' AND vtuber_id IN \
        (SELECT vtuber_id FROM vtubers WHERE retired_at IS NULL OR retired_at + '2 week' >= NOW())"
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channels", query)
}

pub async fn list_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" FROM channels"
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channels", query)
}

pub async fn list_channels_with_stats(
    channel_ids: &[i32],
    pool: &PgPool,
) -> Result<Vec<ChannelWithStats>> {
    let query = sqlx::query_as!(
        ChannelWithStats,
        "SELECT vtuber_id, \
        platform as \"platform: _\", \
        view, view_1d_ago, view_7d_ago, view_30d_ago, \
        subscriber, subscriber_1d_ago, subscriber_7d_ago, subscriber_30d_ago, \
        revenue, revenue_1d_ago, revenue_7d_ago, revenue_30d_ago \
        FROM channels WHERE channel_id = ANY($1)",
        channel_ids,
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channels", query)
}
