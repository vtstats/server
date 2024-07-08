use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{Duration, DurationRound, Utc};
use integration_youtube::{
    data_api::videos::{list_videos, Stream},
    pubsub::Event,
    youtubei::player,
};
use tracing::Span;

use vtstats_database::{
    channels::{get_active_channel_by_platform_id, Platform},
    jobs::{queue_collect_youtube_stream_metadata, queue_send_notification},
    streams::{
        delete_stream, end_stream, get_stream_by_platform_id, StreamStatus, UpsertStreamQuery,
    },
};

use crate::{error::ApiResult, AppContext};

pub async fn publish_content(State(ctx): State<AppContext>, event: Event) -> ApiResult<Response> {
    match event {
        Event::Modification {
            platform_channel_id,
            platform_stream_id,
        } => handle_modification(&platform_channel_id, &platform_stream_id, ctx).await,
        Event::Deletion {
            platform_stream_id, ..
        } => handle_deletion(&platform_stream_id, ctx).await,
    }?;

    Ok(StatusCode::OK.into_response())
}

async fn handle_modification(
    platform_channel_id: &str,
    platform_stream_id: &str,
    ctx: AppContext,
) -> anyhow::Result<()> {
    let channel =
        get_active_channel_by_platform_id(Platform::Youtube, platform_channel_id, &ctx.pool)
            .await?;

    let Some(channel) = channel else {
        tracing::warn!("Cannot find youtube channel of {}", platform_channel_id);
        return Ok(());
    };

    let mut videos = list_videos(platform_stream_id, &ctx.client).await?;

    let stream: Option<Stream> = videos.pop().and_then(Into::into);

    let Some(youtube_stream) = stream else {
        tracing::warn!("Stream not found, platform_stream_id={platform_stream_id}");
        return Ok(());
    };

    let mut thumbnail_url = None;
    if youtube_stream.status != StreamStatus::Ended {
        thumbnail_url = player(platform_stream_id, &ctx.client)
            .await
            .ok()
            .and_then(|res| Some(res.get_thumbnail_url()?.split_once('?')?.0.to_string()));
    }

    let stream_id = UpsertStreamQuery {
        platform: Platform::Youtube,
        vtuber_id: &channel.vtuber_id,
        platform_stream_id: &youtube_stream.id,
        channel_id: channel.channel_id,
        title: &youtube_stream.title,
        status: youtube_stream.status,
        thumbnail_url,
        schedule_time: youtube_stream.schedule_time,
        start_time: youtube_stream.start_time,
        end_time: youtube_stream.end_time,
    }
    .execute(&ctx.pool, &ctx.search)
    .await?;

    Span::current().record("stream_id", stream_id);

    let now = Utc::now();

    match (
        youtube_stream.schedule_time,
        youtube_stream.start_time,
        youtube_stream.end_time,
    ) {
        (Some(time), None, None) | (_, Some(time), None) => {
            queue_collect_youtube_stream_metadata(std::cmp::max(now, time), stream_id, &ctx.pool)
                .await?;
        }
        _ => {}
    }

    let next = now.duration_trunc(Duration::seconds(5))? + Duration::seconds(5);

    queue_send_notification(next, stream_id, &ctx.pool).await?;

    Ok(())
}

async fn handle_deletion(platform_stream_id: &str, ctx: AppContext) -> anyhow::Result<()> {
    let stream =
        get_stream_by_platform_id(Platform::Youtube, platform_stream_id, &ctx.pool).await?;

    let Some(stream) = stream else {
        return Ok(());
    };

    Span::current().record("stream_id", stream.stream_id);

    if stream.status == StreamStatus::Scheduled {
        tracing::warn!("delete schedule stream {}", stream.platform_id);
        delete_stream(stream.stream_id, &ctx.pool, &ctx.search).await?;
    } else {
        end_stream(stream.stream_id, &ctx.pool, &ctx.search).await?;
    }

    Ok(())
}
