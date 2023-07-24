use integration_youtube::pubsub::Event;
use std::convert::Into;

use warp::{http::StatusCode, Rejection};

use vtstat_database::{
    channels::ListChannelsQuery,
    jobs::{JobPayload, PushJobQuery, UpsertYoutubeStreamJobPayload},
    streams::{delete_stream, end_stream, ListYouTubeStreamsQuery, StreamStatus},
    PgPool,
};

use crate::reject::WarpError;

pub async fn publish_content(event: Event, pool: PgPool) -> Result<StatusCode, Rejection> {
    match event {
        Event::Modification {
            platform_channel_id,
            platform_stream_id,
        } => {
            let channels = ListChannelsQuery {
                platform: "youtube",
            }
            .execute(&pool)
            .await
            .map_err(Into::<WarpError>::into)?;

            let channel = channels
                .into_iter()
                .find(|ch| ch.platform_id == platform_channel_id);

            let Some(channel) = channel else {
                return Ok(StatusCode::NOT_FOUND);
            };

            PushJobQuery {
                continuation: None,
                next_run: None,
                payload: JobPayload::UpsertYoutubeStream(UpsertYoutubeStreamJobPayload {
                    channel_id: channel.channel_id,
                    vtuber_id: channel.vtuber_id,
                    platform_stream_id,
                }),
            }
            .execute(&pool)
            .await
            .map_err(Into::<WarpError>::into)?;
        }
        Event::Deletion {
            platform_stream_id, ..
        } => {
            let streams = ListYouTubeStreamsQuery {
                platform_ids: &[platform_stream_id.clone()],
                ..Default::default()
            }
            .execute(&pool)
            .await
            .map_err(Into::<WarpError>::into)?;

            let stream = streams
                .iter()
                .find(|stream| stream.platform_id == platform_stream_id);

            let Some(stream) = stream else {
                return Ok(StatusCode::NOT_FOUND);
            };

            if stream.status == StreamStatus::Scheduled {
                delete_stream(stream.stream_id, &pool).await
            } else {
                end_stream(stream.stream_id, &pool).await
            }
            .map_err(Into::<WarpError>::into)?;
        }
    }

    Ok(StatusCode::OK)
}
