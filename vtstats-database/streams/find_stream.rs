use sqlx::{PgPool, Result};

use super::StreamStatus;

pub struct FindStreamResult {
    pub vtuber_id: String,
    pub status: StreamStatus,
}

pub async fn find_stream(stream_id: i32, pool: &PgPool) -> Result<Option<FindStreamResult>> {
    let query = sqlx::query_as!(
        FindStreamResult,
        "SELECT c.vtuber_id, s.status as \"status: _\" from streams s \
        LEFT JOIN channels c ON s.channel_id = c.channel_id \
        WHERE s.stream_id = $1",
        stream_id
    )
    .fetch_optional(pool);

    crate::otel::execute_query!("SELECT", "streams", query)
}
