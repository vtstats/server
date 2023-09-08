use sqlx::{PgPool, Result};

pub async fn update_stream_title(stream_id: i32, title: String, pool: &PgPool) -> Result<()> {
    let query = sqlx::query!(
        "UPDATE streams SET title = $1 WHERE stream_id = $2",
        title,
        stream_id
    )
    .execute(pool);

    crate::otel::execute_query!("UPDATE", "streams", query)?;

    Ok(())
}
