use sqlx::{PgPool, Result};

use super::Job;

pub struct PullJobQuery {
    pub limit: i64,
}

impl PullJobQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Job>> {
        sqlx::query_as::<_, Job>(
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
                 LIMIT $1
            )
  RETURNING *
            "#,
        )
        .bind(self.limit)
        .fetch_all(pool)
        .await
    }
}

#[cfg(test)]
#[sqlx::test]
async fn test(pool: PgPool) -> Result<()> {
    sqlx::query!("DELETE FROM jobs").execute(&pool).await?;

    sqlx::query!(
        r#"
INSERT INTO jobs (kind, payload, status, next_run)
          VALUES ('upsert_youtube_stream', '{"vtuber_id":"poi","channel_id":0,"platform_stream_id":"foo1"}', 'queued',  NOW() + INTERVAL '15s'),
                 ('upsert_youtube_stream', '{"vtuber_id":"poi","channel_id":0,"platform_stream_id":"foo2"}', 'queued',  NOW() - INTERVAL '15s'),
                 ('upsert_youtube_stream', '{"vtuber_id":"poi","channel_id":0,"platform_stream_id":"foo3"}', 'running', NOW() - INTERVAL '15s'),
                 ('upsert_youtube_stream', '{"vtuber_id":"poi","channel_id":0,"platform_stream_id":"foo4"}', 'success', NOW() - INTERVAL '15s'),
                 ('upsert_youtube_stream', '{"vtuber_id":"poi","channel_id":0,"platform_stream_id":"foo5"}', 'failed',  NOW() - INTERVAL '15s');
        "#
    )
    .execute(&pool)
    .await?;

    let jobs = PullJobQuery { limit: 0 }.execute(&pool).await?;

    assert!(jobs.is_empty());

    let jobs = PullJobQuery { limit: 5 }.execute(&pool).await?;

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
