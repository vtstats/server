use sqlx::{PgPool, Result};

use crate::channels::Platform;

use super::Stream;

pub async fn get_stream_by_platform_id(
    platform: Platform,
    platform_id: &str,
    pool: &PgPool,
) -> Result<Option<Stream>> {
    let query = sqlx::query_as!(
        Stream,
        "SELECT platform as \"platform: _\", \
        platform_id, \
        stream_id, \
        title, \
        channel_id, \
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
        WHERE platform = $1 AND platform_id = $2",
        platform as _,
        platform_id
    )
    .fetch_optional(pool);

    crate::otel::execute_query!("SELECT", "streams", query)
}
