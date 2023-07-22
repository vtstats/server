use sqlx::{PgPool, Result};

pub async fn re_run_job(job_id: i32, pool: &PgPool) -> Result<()> {
    let query = sqlx::query!(
        r#"UPDATE jobs SET status = 'queued', updated_at = NOW() WHERE job_id = $1"#,
        job_id
    )
    .execute(pool);

    crate::otel::instrument("UPDATE", "jobs", query).await?;

    Ok(())
}
