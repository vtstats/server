use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::{Job, JobStatus};

pub async fn list_jobs_order_by_updated_at(
    status: String,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<Job>> {
    let status = match status.as_str() {
        "queued" => JobStatus::Queued,
        "running" => JobStatus::Running,
        "success" => JobStatus::Success,
        "failed" => JobStatus::Failed,
        _ => JobStatus::Queued,
    };

    let query = sqlx::query_as::<_, Job>(
        "SELECT * FROM jobs \
        WHERE status = $1 \
        AND (updated_at < $2 OR $2 is null) \
        ORDER BY updated_at DESC \
        LIMIT 24",
    )
    .bind(status)
    .bind(end_at)
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "jobs", query)
}

// TODO add unit tests
