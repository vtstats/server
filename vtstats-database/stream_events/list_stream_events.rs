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
pub struct ChannelRevenue {
    pub channel_id: i32,
    pub time: DateTime<Utc>,
    pub amount: Option<String>,
    pub symbol: Option<String>,
}

pub async fn list_revenue_by_channel_start_at(
    start_at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenue>> {
    let query = sqlx::query_as!(
        ChannelRevenue,
        "SELECT channel_id, time, \
        (value->>'paid_amount') amount, \
        (value->>'paid_currency_symbol') symbol \
        FROM stream_events LEFT JOIN streams ON \
        streams.stream_id = stream_events.stream_id \
        WHERE time > $1 AND (kind = 'youtube_super_chat' OR kind = 'youtube_super_sticker')",
        start_at
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "stream_events", query)
}

pub async fn list_revenue_by_channel_end_at(
    end_at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<ChannelRevenue>> {
    let query = sqlx::query_as!(
        ChannelRevenue,
        "SELECT channel_id, time, \
        (value->>'paid_amount') amount, \
        (value->>'paid_currency_symbol') symbol \
        FROM stream_events LEFT JOIN streams ON \
        streams.stream_id = stream_events.stream_id \
        WHERE time <= $1 AND (kind = 'youtube_super_chat' OR kind = 'youtube_super_sticker')",
        end_at
    )
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "stream_events", query)
}
