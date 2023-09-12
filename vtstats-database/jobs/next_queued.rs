use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub async fn next_queued(pool: &PgPool) -> Result<Option<DateTime<Utc>>> {
    let query =
        sqlx::query!("SELECT min(next_run) FROM jobs WHERE status = 'queued'").fetch_optional(pool);

    let record = crate::otel::execute_query!("SELECT", "jobs", query)?;

    Ok(record.and_then(|i| i.min))
}
