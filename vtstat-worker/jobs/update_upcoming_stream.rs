use chrono::{Duration, DurationRound, Utc};
use vtstat_database::{
    jobs::{CollectYoutubeStreamMetadataJobPayload, JobPayload, PushJobQuery},
    streams::GetUpcomingStreamsQuery,
    PgPool,
};

use super::JobResult;

pub async fn execute(pool: &PgPool) -> anyhow::Result<JobResult> {
    let d = Duration::seconds(15);
    let next_run = Utc::now().duration_trunc(d).unwrap() + d;

    let streams = GetUpcomingStreamsQuery.execute(&pool).await?;

    for stream in streams {
        PushJobQuery {
            continuation: None,
            next_run: None,
            payload: JobPayload::CollectYoutubeStreamMetadata(
                CollectYoutubeStreamMetadataJobPayload {
                    stream_id: stream.stream_id,
                    platform_channel_id: stream.platform_channel_id,
                    platform_stream_id: stream.platform_stream_id,
                },
            ),
        }
        .execute(pool)
        .await?;
    }

    Ok(JobResult::Next {
        run: next_run,
        continuation: None,
    })
}
