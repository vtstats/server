use meilisearch_sdk::client::Client;
use sqlx::{PgPool, Result};

pub async fn delete_stream(stream_id: i32, pool: &PgPool, client: &Client) -> Result<()> {
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

    tx.commit().await?;

    if let Err(err) = super::meilisearch::delete(stream_id, client).await {
        eprintln!("meili: {err:?}");
    }

    Ok(())
}
