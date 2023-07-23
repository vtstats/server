use chrono::{Duration, DurationRound, Utc};
use vtstat_database::{
    jobs::{
        CollectYoutubeStreamMetadataJobPayload, JobPayload, PushJobQuery,
        SendNotificationJobPayload, UpsertYoutubeStreamJobPayload,
    },
    streams::{StreamStatus as StreamStatus_, UpsertYouTubeStreamQuery},
    PgPool,
};
use vtstat_request::{RequestHub, StreamStatus};

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    hub: RequestHub,
    payload: UpsertYoutubeStreamJobPayload,
) -> anyhow::Result<JobResult> {
    let UpsertYoutubeStreamJobPayload {
        channel_id,
        platform_stream_id,
        vtuber_id,
    } = payload;

    let mut streams = hub.youtube_streams(&[platform_stream_id.clone()]).await?;

    let Some(youtube_stream) = streams.first_mut() else {
        anyhow::bail!("Stream not found, platform_stream_id={platform_stream_id}");
    };

    let thumbnail_url = hub.upload_thumbnail(&youtube_stream.id).await;

    let status = match youtube_stream.status {
        StreamStatus::Ended => StreamStatus_::Ended,
        StreamStatus::Live => StreamStatus_::Live,
        StreamStatus::Scheduled => StreamStatus_::Scheduled,
    };

    let stream_id = UpsertYouTubeStreamQuery {
        platform_stream_id: &youtube_stream.id,
        channel_id,
        title: &youtube_stream.title,
        status,
        thumbnail_url,
        schedule_time: youtube_stream.schedule_time,
        start_time: youtube_stream.start_time,
        end_time: youtube_stream.end_time,
    }
    .execute(pool)
    .await?;

    let now = Utc::now();

    match (
        youtube_stream.schedule_time,
        youtube_stream.start_time,
        youtube_stream.end_time,
    ) {
        (Some(time), None, None) | (_, Some(time), None) => {
            let next = std::cmp::max(now, time);

            PushJobQuery {
                continuation: None,
                next_run: Some(next),
                payload: JobPayload::CollectYoutubeStreamMetadata(
                    CollectYoutubeStreamMetadataJobPayload {
                        stream_id,
                        platform_stream_id,
                        platform_channel_id: youtube_stream.channel_id.to_owned(),
                    },
                ),
            }
            .execute(pool)
            .await?;
        }
        _ => {}
    }

    let next = now.duration_trunc(Duration::seconds(5)).unwrap() + Duration::seconds(5);

    PushJobQuery {
        continuation: None,
        next_run: Some(next),
        payload: JobPayload::SendNotification(SendNotificationJobPayload {
            stream_platform: "youtube".into(),
            stream_platform_id: youtube_stream.id.clone(),
            vtuber_id: vtuber_id.clone(),
        }),
    }
    .execute(pool)
    .await?;

    Ok(JobResult::Completed)
}
