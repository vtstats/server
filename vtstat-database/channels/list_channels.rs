use sqlx::{PgPool, Result};

use super::Channel;

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
