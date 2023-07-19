use anyhow::bail;
use chrono::{Duration, Utc};
use vtstat_database::{
    jobs::{
        CollectYoutubeStreamLiveChatJobPayload, CollectYoutubeStreamMetadataJobPayload, JobPayload,
        PushJobQuery,
    },
    stream_stats::AddStreamViewerStatsQuery,
    streams::{EndStreamQuery, StartStreamQuery},
    PgPool,
};
use vtstat_request::RequestHub;

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    hub: RequestHub,
    continuation: Option<String>,
    payload: CollectYoutubeStreamMetadataJobPayload,
) -> anyhow::Result<JobResult> {
    let CollectYoutubeStreamMetadataJobPayload {
        stream_id,
        platform_stream_id,
        platform_channel_id,
    } = payload;

    let response = match continuation {
        Some(ref cont) => hub.updated_metadata_with_continuation(cont).await,
        None => hub.updated_metadata(&platform_stream_id).await,
    }?;

    match (
        response.timeout(),
        response.continuation(),
        response.is_waiting(),
    ) {
        // stream not found
        (None, _, _) | (_, None, _) => {
            EndStreamQuery {
                stream_id,
                end_time: Some(Utc::now()),
                ..Default::default()
            }
            .execute(pool)
            .await?;

            Ok(JobResult::Completed {})
        }

        // stream was ended
        (Some(timeout), _, false) if timeout.as_secs() > 5 => {
            let time = Utc::now();

            let mut streams = hub
                .youtube_streams(&[platform_stream_id.to_string()])
                .await?;

            let Some(stream) = streams.first_mut() else {
                bail!("Stream not found: platform_stream_id: {platform_stream_id}");
            };

            EndStreamQuery {
                stream_id,
                updated_at: Some(time),
                title: Some(&*stream.title),
                end_time: stream.end_time,
                start_time: stream.start_time,
                schedule_time: stream.schedule_time,
                likes: stream.likes,
            }
            .execute(pool)
            .await?;

            if let Some(count) = stream.viewers {
                AddStreamViewerStatsQuery {
                    stream_id,
                    count,
                    time,
                }
                .execute(pool)
                .await?;
            }

            Ok(JobResult::Completed)
        }

        // stream is still waiting
        (Some(timeout), Some(continuation), true) => Ok(JobResult::Next {
            run: Utc::now() + Duration::from_std(timeout)?,
            continuation: Some(continuation.to_string()),
        }),

        // stream is on air
        (Some(timeout), Some(continuation), false) => {
            let time = Utc::now();

            StartStreamQuery {
                stream_id,
                title: response.title().as_deref(),
                start_time: time,
                likes: response.like_count(),
            }
            .execute(pool)
            .await?;

            if let Some(viewer) = response.view_count() {
                AddStreamViewerStatsQuery {
                    time,
                    count: viewer,
                    stream_id,
                }
                .execute(pool)
                .await?;
            }

            PushJobQuery {
                continuation: None,
                next_run: None,
                payload: JobPayload::CollectYoutubeStreamLiveChat(
                    CollectYoutubeStreamLiveChatJobPayload {
                        stream_id,
                        platform_stream_id,
                        platform_channel_id,
                    },
                ),
            }
            .execute(pool)
            .await?;

            Ok(JobResult::Next {
                run: time + Duration::from_std(timeout)?,
                continuation: Some(continuation.to_string()),
            })
        }
    }
}
