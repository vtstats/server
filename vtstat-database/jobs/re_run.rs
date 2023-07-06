use sqlx::{PgPool, Result};

pub async fn re_run_job(job_id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        r#"UPDATE jobs SET status = 'queued', updated_at = NOW() WHERE job_id = $1"#,
        job_id
    )
    .execute(pool)
    .await
    .map(|_| ())
}
