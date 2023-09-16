use sqlx::{PgPool, Result};

use super::Job;

pub async fn pull_jobs(pool: &PgPool) -> Result<Vec<Job>> {
    let query = sqlx::query_as::<_, Job>(
        r#"
   UPDATE jobs
      SET status = 'running'
    WHERE job_id IN
          (
               SELECT job_id
               FROM jobs
               WHERE (next_run <= NOW() OR next_run IS NULL)
               AND status = 'queued'
               ORDER BY next_run
               FOR UPDATE SKIP LOCKED
          )
RETURNING *
        "#,
    )
    .fetch_all(pool);

    crate::otel::execute_query!("UPDATE", "jobs", query)
}

#[cfg(test)]
#[sqlx::test]
async fn test(pool: PgPool) -> Result<()> {
    sqlx::query!("DELETE FROM jobs").execute(&pool).await?;

    sqlx::query!(
        r#"
INSERT INTO jobs (kind, payload, status, next_run)
          VALUES ('collect_twitch_stream_metadata', '{"stream_id":1}', 'queued',  NOW() + INTERVAL '15s'),
                 ('collect_twitch_stream_metadata', '{"stream_id":2}', 'queued',  NOW() - INTERVAL '15s'),
                 ('collect_twitch_stream_metadata', '{"stream_id":3}', 'running', NOW() - INTERVAL '15s'),
                 ('collect_twitch_stream_metadata', '{"stream_id":4}', 'success', NOW() - INTERVAL '15s'),
                 ('collect_twitch_stream_metadata', '{"stream_id":5}', 'failed',  NOW() - INTERVAL '15s');
        "#
    )
    .execute(&pool)
    .await?;

    let jobs = pull_jobs(&pool).await?;

    assert_eq!(jobs.len(), 1);
    assert_eq!(
        sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE status = 'running'")
            .fetch_all(&pool)
            .await?
            .len(),
        2
    );

    Ok(())
}
