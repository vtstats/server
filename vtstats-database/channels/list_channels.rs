use sqlx::{PgPool, Result};

use crate::channels::ChannelWithStats;

use super::{Channel, Platform};

pub async fn list_active_channels_by_platform(
    platform: Platform,
    pool: &PgPool,
) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE platform = $1 AND vtuber_id IN \
        (SELECT vtuber_id FROM vtubers WHERE retired_at IS NULL OR retired_at + '2 week' >= NOW())",
        platform as _
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channels", query)
}

pub async fn get_channel_by_id(channel_id: i32, pool: &PgPool) -> Result<Option<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE channel_id = $1",
        channel_id
    )
    .fetch_optional(pool);

    crate::otel::execute_query!("SELECT", "channels", query)
}

pub async fn get_active_channel_by_platform_id(
    platform: Platform,
    platform_id: &str,
    pool: &PgPool,
) -> Result<Option<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE platform = $1 AND platform_id = $2 AND vtuber_id IN \
        (SELECT vtuber_id FROM vtubers WHERE retired_at IS NULL OR retired_at + '2 week' >= NOW())",
        platform as _,
        platform_id
    )
    .fetch_optional(pool);

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
