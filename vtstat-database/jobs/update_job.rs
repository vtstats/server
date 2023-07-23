use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::{Job, JobStatus};

pub struct UpdateJobQuery {
    pub job_id: i32,
    pub status: JobStatus,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub continuation: Option<String>,
}

impl UpdateJobQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Job> {
        let query = sqlx::query_as::<_, Job>(
            r#"
     UPDATE jobs
        SET status       = $1,
            next_run     = $2,
            last_run     = $5,
            continuation = $3,
            updated_at   = NOW()
      WHERE job_id       = $4
  RETURNING *
            "#,
        )
        .bind(self.status) // $1
        .bind(self.next_run) // $2
        .bind(self.continuation) // $3
        .bind(self.job_id) // $4
        .bind(self.last_run) // $5
        .fetch_one(pool);

        crate::otel::instrument("UPDATE", "jobs", query).await
    }
}

// TODO add unit tests
