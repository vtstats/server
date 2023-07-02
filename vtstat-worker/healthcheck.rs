use std::{env, time::Duration};

use vtstat_database::{
    jobs::{JobKind, JobPayload, JobStatus, ListJobsQuery, PushJobQuery},
    PgPool,
};

pub async fn healthcheck() -> anyhow::Result<()> {
    vtstat_utils::dotenv::load();

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    PushJobQuery {
        continuation: None,
        next_run: None,
        payload: JobPayload::HealthCheck,
    }
    .execute(&pool)
    .await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    let jobs = ListJobsQuery {
        kind: JobKind::HealthCheck,
    }
    .execute(&pool)
    .await?;

    if jobs.is_empty() {
        anyhow::bail!("Health check failed: job not found");
    }

    let status = &jobs[0].status;

    if *status != JobStatus::Success {
        anyhow::bail!("Health check failed: job is {status:?}");
    }

    Ok(())
}
