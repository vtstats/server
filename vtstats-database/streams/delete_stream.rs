use sqlx::{PgPool, Result};

pub async fn delete_stream(stream_id: i32, pool: &PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;

    let query =
        sqlx::query!("DELETE from stream_events WHERE stream_id = $1", stream_id).execute(&mut *tx);

    crate::otel::execute_query!("DELETE", "stream_events", query)?;

    let query = sqlx::query!(
        "DELETE from stream_chat_stats WHERE stream_id = $1",
        stream_id
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("DELETE", "stream_chat_stats", query)?;

    let query = sqlx::query!(
        "DELETE from stream_viewer_stats WHERE stream_id = $1",
        stream_id
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("DELETE", "stream_viewer_stats", query)?;

    let query =
        sqlx::query!("DELETE from streams WHERE stream_id = $1", stream_id).execute(&mut *tx);

    crate::otel::execute_query!("DELETE", "streams", query)?;

    tx.commit().await
}
