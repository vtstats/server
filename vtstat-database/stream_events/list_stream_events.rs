use sqlx::{PgPool, Result};

use super::StreamEvent;

pub async fn list_stream_events(stream_id: i32, pool: &PgPool) -> Result<Vec<StreamEvent>> {
    let query = sqlx::query_as::<_, StreamEvent>(
        r#"SELECT kind, time, value FROM stream_events WHERE stream_id = $1"#,
    )
    .bind(stream_id)
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "stream_events", query).await
}
