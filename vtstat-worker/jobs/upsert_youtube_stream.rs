use vtstat_database::{
    jobs::UpsertYoutubeStreamJobPayload,
    streams::{StreamStatus as StreamStatus_, UpsertYouTubeStreamQuery},
    PgPool,
};
use vtstat_request::{RequestHub, StreamStatus};

use super::JobResult;

pub async fn execute(
    pool: PgPool,
    hub: RequestHub,
    payload: UpsertYoutubeStreamJobPayload,
) -> anyhow::Result<JobResult> {
    let UpsertYoutubeStreamJobPayload {
        channel_id,
        platform_stream_id,
    } = payload;

    let mut streams = hub.youtube_streams(&[platform_stream_id.clone()]).await?;

    let Some(stream) = streams.first_mut() else {
        anyhow::bail!("Stream not found, platform_stream_id={platform_stream_id}");
    };

    let thumbnail_url = hub.upload_thumbnail(&stream.id).await;

    UpsertYouTubeStreamQuery {
        platform_stream_id: &stream.id,
        channel_id,
        title: &stream.title,
        status: match stream.status {
            StreamStatus::Ended => StreamStatus_::Ended,
            StreamStatus::Live => StreamStatus_::Live,
            StreamStatus::Scheduled => StreamStatus_::Scheduled,
        },
        thumbnail_url,
        schedule_time: stream.schedule_time,
        start_time: stream.start_time,
        end_time: stream.end_time,
    }
    .execute(&pool)
    .await?;

    Ok(JobResult::Completed)
}
