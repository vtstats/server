use chrono::Utc;
use reqwest::header::CONTENT_TYPE;
use vtstats_database::{
    channels::{list_twitch_channels, Platform},
    jobs::queue_collect_twitch_stream_metadata,
    streams::{StreamStatus, UpsertStreamQuery},
    PgPool,
};
use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{Reply, Response},
    Filter,
};

use integration_twitch::{validate, Event, Notification};

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
) -> Result<Response, Rejection> {
    match notification {
        Notification::Event(event) => match event {
            Event::ChannelUpdateEvent(event) => {
                tracing::info!("twitch channel.update: {:?}", event);
                Ok(StatusCode::NO_CONTENT.into_response())
            }
            Event::StreamOnlineEvent(event) => {
                tracing::info!("twitch stream.online: {:?}", event);

                let channels = list_twitch_channels(&pool).await.map_err(WarpError::from)?;

                let channel = channels
                    .iter()
                    .find(|ch| ch.platform_id == event.broadcaster_user_id);

                if let Some(channel) = channel {
                    let stream_id = UpsertStreamQuery {
                        vtuber_id: &channel.vtuber_id,
                        platform: Platform::Twitch,
                        platform_stream_id: &event.id,
                        channel_id: channel.channel_id,
                        title: &format!("Twitch stream #{}", event.broadcaster_user_login),
                        status: StreamStatus::Live,
                        thumbnail_url: None,
                        schedule_time: None,
                        start_time: Some(event.started_at),
                        end_time: None,
                    }
                    .execute(&pool)
                    .await
                    .map_err(WarpError::from)?;

                    queue_collect_twitch_stream_metadata(
                        Utc::now(),
                        stream_id,
                        event.id,
                        event.broadcaster_user_id,
                        event.broadcaster_user_login,
                        &pool,
                    )
                    .await
                    .map_err(WarpError::from)?;
                } else {
                    tracing::warn!(
                        "Cannot find twitch channel of #{}",
                        event.broadcaster_user_login
                    );
                }

                Ok(StatusCode::NO_CONTENT.into_response())
            }
            Event::StreamOfflineEvent(event) => {
                tracing::info!("twitch stream.offline: {:?}", event);
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
