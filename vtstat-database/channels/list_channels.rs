use sqlx::{PgPool, Result};

use crate::channels::ChannelWithStats;

use super::{Channel, Platform};

pub async fn list_youtube_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE platform = 'youtube'"
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channels", query).await
}

pub async fn list_bilibili_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" \
        FROM channels WHERE platform = 'bilibili'"
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channels", query).await
}

pub async fn list_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let query = sqlx::query_as!(
        Channel,
        "SELECT channel_id, platform_id, vtuber_id, platform as \"platform: _\" FROM channels"
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channels", query).await
}

pub async fn list_channels_with_stats(
    vtuber_ids: &[String],
    platform: Platform,
    pool: &PgPool,
) -> Result<Vec<ChannelWithStats>> {
    let query = sqlx::query_as!(
        ChannelWithStats,
        "SELECT vtuber_id, \
        platform as \"platform: _\", \
        view, view_1d_ago, view_7d_ago, view_30d_ago, \
        subscriber, subscriber_1d_ago, subscriber_7d_ago, subscriber_30d_ago, \
        revenue, revenue_1d_ago, revenue_7d_ago, revenue_30d_ago \
        FROM channels \
        WHERE vtuber_id = ANY($1) \
        AND platform = $2",
        vtuber_ids,
        platform as _,
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channels", query).await
}
