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

// TODO add unit tests
