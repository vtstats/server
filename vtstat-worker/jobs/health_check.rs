use super::JobResult;

pub async fn execute() -> anyhow::Result<JobResult> {
    Ok(JobResult::Completed)
}
