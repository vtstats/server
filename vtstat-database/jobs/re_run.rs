use sqlx::{PgPool, Result};

pub async fn re_run_job(job_id: i32, pool: &PgPool) -> Result<()> {
    let query = sqlx::query!(
        "UPDATE jobs SET \
        status = 'queued', \
        next_run = NOW(), \
        updated_at = NOW() \
        WHERE job_id = $1",
        job_id
    )
    .execute(pool);

    crate::otel::instrument("UPDATE", "jobs", query).await?;

    let _ = sqlx::query!("SELECT pg_notify('vt_new_job_queued', '10000000000000')")
        .execute(pool)
        .await
        .map_err(|err| {
            tracing::error!("push job: {err:?}");
        });

    Ok(())
}
