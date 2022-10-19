use vtstat_database::PgPool;

use super::JobResult;
use crate::timer::{timer, Calendar};

pub async fn execute(pool: &PgPool) -> anyhow::Result<JobResult> {
    let (_, next_run) = timer(Calendar::Daily);

    Ok(JobResult::Next {
        run: next_run,
        continuation: None,
    })
}
