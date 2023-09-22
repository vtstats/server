use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::StreamEvent;

pub async fn list_stream_events(stream_id: i32, pool: &PgPool) -> Result<Vec<StreamEvent>> {
    let query = sqlx::query_as::<_, StreamEvent>(
        r#"SELECT kind, time, value FROM stream_events WHERE stream_id = $1"#,
    )
    .bind(stream_id)
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "stream_events", query)
}

#[derive(Debug)]
pub struct ChannelRevenueEvent {
    pub channel_id: i32,
    pub time: DateTime<Utc>,
    pub amount: Option<String>,
    pub symbol: Option<String>,
}

pub async fn list_youtube_channel_revenue_events(
    start_at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenueEvent>> {
    let query = sqlx::query_as!(
        ChannelRevenueEvent,
        "SELECT channel_id, time, \
        (value->>'paid_amount') amount, \
        (value->>'paid_currency_symbol') symbol \
        FROM stream_events LEFT JOIN streams ON streams.stream_id = stream_events.stream_id \
        WHERE time > $1 AND (kind = 'youtube_super_chat' OR kind = 'youtube_super_sticker')",
        start_at
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "stream_events", query)
}

pub async fn list_twitch_channel_revenue_events(
    start_at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenueEvent>> {
    let query = sqlx::query_as!(
        ChannelRevenueEvent,
        "SELECT channel_id, time, \
        CASE WHEN kind = 'twitch_cheering' THEN 'cheering' ELSE (value->>'currency_code') END symbol, \
        CASE WHEN kind = 'twitch_cheering' THEN (value->>'bits') ELSE (value->>'amount') END amount \
        FROM stream_events LEFT JOIN streams ON streams.stream_id = stream_events.stream_id \
        WHERE time > $1 AND (kind = 'twitch_cheering' OR kind = 'twitch_hyper_chat')",
        start_at
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "stream_events", query)
}

pub async fn list_channel_revenue_events(
    channel_id: i32,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenueEvent>> {
    let query = sqlx::query_as!(
        ChannelRevenueEvent,
        "SELECT channel_id, time, \
        CASE WHEN kind = 'twitch_cheering' THEN 'cheering' \
        WHEN kind = 'twitch_hyper_chat' THEN (value->>'currency_code') \
        ELSE (value->>'paid_currency_symbol') END symbol, \
        CASE WHEN kind = 'twitch_cheering' THEN (value->>'bits') \
        WHEN kind = 'twitch_hyper_chat' THEN (value->>'amount') \
        ELSE (value->>'paid_amount') END amount \
        FROM stream_events LEFT JOIN streams ON streams.stream_id = stream_events.stream_id \
        WHERE channel_id = $1 AND (kind = 'youtube_super_chat' OR kind = 'youtube_super_sticker' \
        OR kind = 'twitch_cheering' OR kind = 'twitch_hyper_chat')",
        channel_id
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "stream_events", query)
}
