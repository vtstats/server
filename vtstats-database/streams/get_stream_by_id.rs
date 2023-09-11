use sqlx::{PgPool, Result};

use crate::streams::Stream;

pub async fn get_stream_by_id(stream_id: i32, pool: &PgPool) -> Result<Option<Stream>> {
    let query = sqlx::query_as!(
        Stream,
        "SELECT platform as \"platform: _\", \
        platform_id, \
        stream_id, \
        channel_id, \
        title, \
        null as highlighted_title, \
        vtuber_id, \
        thumbnail_url, \
        schedule_time, \
        start_time, \
        end_time, \
        viewer_max, \
        viewer_avg, \
        like_max, \
        updated_at, \
        status as \"status: _\" \
        FROM streams \
        WHERE stream_id = $1",
        stream_id
    )
    .fetch_optional(pool);

    crate::otel::execute_query!("SELECT", "streams", query)
}
