use sqlx::{PgPool, Result};

pub async fn alert_vtuber_id<'a>(before: &str, after: &str, pool: PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;

    let query = sqlx::query!(
        "INSERT INTO vtubers (vtuber_id, native_name, english_name, japanese_name, twitter_username, thumbnail_url) \
        SELECT $1, native_name, english_name, japanese_name, twitter_username, thumbnail_url \
        FROM vtubers WHERE vtuber_id = $2",
        after,
        before
    ).execute(&mut *tx);

    let result = crate::otel::execute_query!("INSERT", "vtubers", query)?;

    if result.rows_affected() == 0 {
        tracing::warn!("VTuber id {before:?} not found");
        return Ok(());
    }

    let query = sqlx::query!(
        "UPDATE channels SET vtuber_id = $1 WHERE vtuber_id = $2",
        after,
        before
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("UPDATE", "vtubers", query)?;

    let query = sqlx::query!(
        "UPDATE streams SET vtuber_id = $1 WHERE vtuber_id = $2",
        after,
        before
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("UPDATE", "vtubers", query)?;

    let value = serde_json::json!({"vtuber_id": after});

    let query = sqlx::query!(
        "UPDATE jobs SET payload = payload || $1 WHERE (payload->>'vtuber_id') = $2",
        value,
        before
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("UPDATE", "jobs", query)?;

    let query = sqlx::query!(
        "UPDATE subscriptions SET payload = payload || $1 WHERE (payload->>'vtuber_id') = $2",
        value,
        before
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("UPDATE", "subscriptions", query)?;

    let query = sqlx::query!(
        "UPDATE notifications SET payload = payload || $1 WHERE (payload->>'vtuber_id') = $2",
        value,
        before
    )
    .execute(&mut *tx);

    crate::otel::execute_query!("UPDATE", "notifications", query)?;

    let query = sqlx::query!("DELETE FROM vtubers WHERE vtuber_id = $1", before).execute(&mut *tx);

    crate::otel::execute_query!("DELETE", "vtubers", query)?;

    tx.commit().await
}
