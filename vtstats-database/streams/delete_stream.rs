use sqlx::{PgPool, Result};

pub async fn delete_stream(stream_id: i32, pool: &PgPool) -> Result<()> {
    let query = sqlx::query!("DELETE from streams WHERE stream_id = $1", stream_id).execute(pool);

    crate::otel::execute_query!("DELETE", "streams", query)?;

    Ok(())
}
