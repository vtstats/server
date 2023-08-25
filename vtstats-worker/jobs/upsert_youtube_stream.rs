use chrono::{Duration, DurationRound, Utc};
use reqwest::Client;

use integration_s3::upload_file;
use integration_youtube::{
    data_api::videos::{list_videos, Stream},
    thumbnail::get_thumbnail,
};
use vtstats_database::{
    jobs::{
        queue_collect_youtube_stream_metadata, queue_send_notification,
        UpsertYoutubeStreamJobPayload,
    },
    streams::UpsertYouTubeStreamQuery,
    PgPool,
};

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    client: Client,
    payload: UpsertYoutubeStreamJobPayload,
) -> anyhow::Result<JobResult> {
    let UpsertYoutubeStreamJobPayload {
        channel_id,
        platform_stream_id,
        vtuber_id,
    } = payload;

    let mut videos = list_videos(&platform_stream_id, &client).await?;

    let stream: Option<Stream> = videos.pop().and_then(Into::into);

    let Some(youtube_stream) = stream else {
        tracing::warn!("Stream not found, platform_stream_id={platform_stream_id}");
        return Ok(JobResult::Completed);
    };

    let thumbnail_url = match get_thumbnail(&youtube_stream.id, &client).await {
        Ok((filename, content_type, bytes)) => {
            match upload_file(&filename, bytes, &content_type, &client).await {
                Ok(thumbnail_url) => Some(thumbnail_url),
                Err(err) => {
                    tracing::error!(exception.stacktrace = ?err, message= %err);
                    None
                }
            }
        }
        Err(err) => {
            tracing::error!(exception.stacktrace = ?err, message= %err);
            None
        }
    };

    let stream_id = UpsertYouTubeStreamQuery {
        vtuber_id: &vtuber_id,
        platform_stream_id: &youtube_stream.id,
        channel_id,
        title: &youtube_stream.title,
        status: youtube_stream.status,
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
            queue_collect_youtube_stream_metadata(
                std::cmp::max(now, time),
                stream_id,
                platform_stream_id,
                youtube_stream.channel_id.to_owned(),
                pool,
            )
            .await?;
        }
        _ => {}
    }

    let next = now.duration_trunc(Duration::seconds(5)).unwrap() + Duration::seconds(5);

    queue_send_notification(
        next,
        "youtube".into(),
        youtube_stream.id.clone(),
        vtuber_id.clone(),
        pool,
    )
    .await?;

    Ok(JobResult::Completed)
}
