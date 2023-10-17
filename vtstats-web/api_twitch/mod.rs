use chrono::{DateTime, Utc};
use integration_s3::upload_file;
use reqwest::{header::CONTENT_TYPE, Client};
use tracing::Span;
use vtstats_database::{
    channels::{get_active_channel_by_platform_id, Platform},
    jobs::queue_collect_twitch_stream_metadata,
    streams::{end_twitch_stream, StreamStatus, UpsertStreamQuery},
    PgPool,
};
use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{Reply, Response},
    Filter,
};

use integration_twitch::{gql::stream_metadata, validate, Event, Notification};

use crate::reject::WarpError;

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("twitch")
        .and(warp::post())
        .and(validate())
        .and_then(move |noti| twitch_notification(noti, pool.clone()))
}

async fn twitch_notification(
    notification: Notification,
    pool: PgPool,
) -> anyhow::Result<Response, Rejection> {
    match notification {
        Notification::Event(event) => match event {
            Event::ChannelUpdateEvent(event) => {
                tracing::info!("twitch channel.update: {:?}", event);
                Ok(StatusCode::NO_CONTENT.into_response())
            }
            Event::StreamOnlineEvent(event) => {
                tracing::info!("twitch stream.online: {:?}", event);

                handle_stream_online(
                    event.broadcaster_user_id,
                    event.broadcaster_user_login,
                    event.id,
                    event.started_at,
                    &pool,
                )
                .await
                .map_err(WarpError)?;

                Ok(StatusCode::NO_CONTENT.into_response())
            }
            Event::StreamOfflineEvent(event) => {
                tracing::info!("twitch stream.offline: {:?}", event);

                handle_stream_offline(
                    event.broadcaster_user_id,
                    event.broadcaster_user_login,
                    &pool,
                )
                .await
                .map_err(WarpError)?;

                Ok(StatusCode::NO_CONTENT.into_response())
            }
        },
        Notification::Verification(challenge) => Ok(warp::reply::with_status(
            warp::reply::with_header(challenge.challenge, CONTENT_TYPE, "text/plain"),
            StatusCode::OK,
        )
        .into_response()),
        Notification::Revocation(subscription) => {
            tracing::info!("twitch revocation: {:?}", subscription);
            Ok(StatusCode::NO_CONTENT.into_response())
        }
    }
}

async fn handle_stream_online(
    platform_channel_id: String,
    platform_channel_login: String,
    platform_stream_id: String,
    stream_start_time: DateTime<Utc>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let client = vtstats_utils::reqwest::new()?;

    let channel =
        get_active_channel_by_platform_id(Platform::Twitch, &platform_channel_id, pool).await?;

    let Some(channel) = channel else {
        tracing::warn!("Cannot find twitch channel of #{}", platform_channel_login);
        return Ok(());
    };

    let metadata = stream_metadata(&platform_channel_login, &client).await?;

    let Some(platform_stream) = metadata.data.user.stream else {
        return Ok(());
    };

    if platform_stream.id != platform_stream_id {
        return Ok(());
    }

    let title = metadata
        .data
        .user
        .last_broadcast
        .title
        .filter(|_| matches!(metadata.data.user.last_broadcast.id, Some(id) if id == platform_stream.id))
        .unwrap_or_else(|| format!("Twitch stream #{}", platform_channel_login));

    let stream_id = UpsertStreamQuery {
        vtuber_id: &channel.vtuber_id,
        platform: Platform::Twitch,
        platform_stream_id: &platform_stream_id,
        channel_id: channel.channel_id,
        title: &title,
        status: StreamStatus::Live,
        thumbnail_url: Some(format!(
            "https://static-cdn.jtvnw.net/previews-ttv/live_user_{}-1280x720.jpg",
            platform_channel_login
        )),
        schedule_time: None,
        start_time: Some(stream_start_time),
        end_time: None,
    }
    .execute(pool)
    .await?;

    Span::current().record("stream_id", stream_id);

    queue_collect_twitch_stream_metadata(Utc::now(), stream_id, pool).await?;

    Ok(())
}

async fn handle_stream_offline(
    platform_channel_id: String,
    platform_channel_login: String,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let client = vtstats_utils::reqwest::new()?;

    let channel =
        get_active_channel_by_platform_id(Platform::Twitch, &platform_channel_id, pool).await?;

    let Some(channel) = channel else {
        tracing::warn!("Cannot find twitch channel of #{}", platform_channel_login);
        return Ok(());
    };

    let thumbnail_url = match get_thumbnail_url(&platform_channel_login, &client).await {
        Ok(url) => Some(url),
        Err(err) => {
            tracing::warn!("Failed to get thumbnail url of #{}", platform_channel_login);
            tracing::warn!("{err:?}");
            None
        }
    };

    end_twitch_stream(channel.channel_id, thumbnail_url, pool).await?;

    Ok(())
}

async fn get_thumbnail_url(
    platform_channel_login: &str,
    client: &Client,
) -> anyhow::Result<String> {
    let res = client
        .get(format!(
            "https://static-cdn.jtvnw.net/previews-ttv/live_user_{}-1280x720.jpg",
            platform_channel_login
        ))
        .send()
        .await?;

    let bytes = res.bytes().await?;

    let now = Utc::now().timestamp();

    upload_file(
        &format!("twitch-{}-{}.jpg", platform_channel_login, now),
        bytes,
        "image/jpeg",
        client,
    )
    .await
}
