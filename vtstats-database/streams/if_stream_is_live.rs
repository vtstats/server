use sqlx::{PgPool, Result};

pub async fn if_stream_is_live(stream_id: i32, pool: &PgPool) -> Result<bool> {
    let query = sqlx::query!(
        "SELECT status = 'live'::stream_status AS is_online FROM streams WHERE stream_id = $1",
        stream_id
    )
    .fetch_optional(pool);

    let record = crate::otel::execute_query!("SELECT", "streams", query)?;

    Ok(record
        .and_then(|record| record.is_online)
        .unwrap_or_default())
}
