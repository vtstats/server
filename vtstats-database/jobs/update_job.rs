use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::JobStatus;

pub struct UpdateJobQuery {
    pub job_id: i32,
    pub status: JobStatus,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
}

impl UpdateJobQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<()> {
        let query = sqlx::query!(
            "UPDATE jobs SET status = $1, next_run = $2, last_run = $4, updated_at = NOW() WHERE job_id = $3",
            self.status as _, // $1
            self.next_run,    // $2
            self.job_id,      // $3
            self.last_run,    // $4
        )
        .execute(pool);

        crate::otel::execute_query!("UPDATE", "jobs", query)?;

        if let (JobStatus::Queued, Some(next_run)) = (self.status, self.next_run) {
            let query = sqlx::query!(
                "SELECT pg_notify('vt_new_job_queued', $1)",
                next_run.timestamp_millis().to_string()
            )
            .execute(pool);

            crate::otel::execute_query!("SELECT", "pg_notify", query)?;
        }

        Ok(())
    }
}

// TODO add unit tests
