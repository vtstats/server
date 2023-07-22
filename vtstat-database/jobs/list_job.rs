use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::{Job, JobKind};

pub struct ListJobsQuery {
    pub kind: JobKind,
}

impl ListJobsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Job>> {
        let query = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE kind = $1")
            .bind(self.kind)
            .fetch_all(pool);

        crate::otel::instrument("SELECT", "jobs", query).await
    }
}

pub async fn list_jobs_order_by_updated_at(
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<Job>> {
    let query = sqlx::query_as::<_, Job>(
        "SELECT * FROM jobs \
        WHERE (updated_at >= $1 or $1 is null) \
        ORDER BY updated_at DESC \
        LIMIT 24",
    )
    .bind(end_at)
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "jobs", query).await
}

// TODO add unit tests
