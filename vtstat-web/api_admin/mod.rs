use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use warp::{reply::Response, Filter, Rejection, Reply};

use integration_admin::{validate, GoogleCerts};
use integration_youtube::youtubei;
use vtstat_database::{
    channels::{CreateChannel, ListChannelsQuery, Platform},
    streams::{Column, ListYouTubeStreamsQuery, Ordering},
    vtubers::CreateVTuber,
    PgPool,
};

use crate::{filters::with_pool, reject::WarpError};

pub fn routes(pool: PgPool) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    let certs = GoogleCerts::new();

    let jobs_api = warp::path!("admin" / "jobs")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::query())
        .and_then(list_jobs);

    let streams_api = warp::path!("admin" / "streams")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::query())
        .and_then(list_streams);

    let notifications_api = warp::path!("admin" / "notifications")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_notifications);

    let vtubers_api = warp::path!("admin" / "vtubers")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_vtubers);

    let subscriptions_api = warp::path!("admin" / "subscriptions")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_subscriptions);

    let channels_api = warp::path!("admin" / "channels")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_channels);

    let re_run_job_api = warp::path!("admin" / "re_run_job" / i32)
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(re_run_job);

    let create_vtuber_api = warp::path!("admin" / "vtuber")
        .and(warp::put())
        .and(validate(certs.clone()))
        .and(with_pool(pool))
        .and(warp::body::json())
        .and_then(create_vtuber);

    jobs_api
        .or(streams_api)
        .or(notifications_api)
        .or(vtubers_api)
        .or(subscriptions_api)
        .or(channels_api)
        .or(re_run_job_api)
        .or(create_vtuber_api)
}

async fn list_jobs(pool: PgPool, parameter: ListStreamsParameter) -> Result<Response, Rejection> {
    let jobs = vtstat_database::jobs::list_jobs_order_by_updated_at(parameter.end_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&jobs).into_response())
}

#[derive(Deserialize)]
pub struct ListStreamsParameter {
    #[serde(default, with = "ts_seconds_option")]
    end_at: Option<DateTime<Utc>>,
}

async fn list_streams(
    pool: PgPool,
    parameter: ListStreamsParameter,
) -> Result<Response, Rejection> {
    let streams = ListYouTubeStreamsQuery {
        limit: Some(24),
        order_by: Some((Column::UpdatedAt, Ordering::Desc)),
        end_at: parameter.end_at.as_ref().map(|dt| (Column::UpdatedAt, dt)),
        ..Default::default()
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}

async fn list_channels(pool: PgPool) -> Result<Response, Rejection> {
    let channels = ListChannelsQuery {
        platform: "youtube",
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&channels).into_response())
}

async fn list_notifications(pool: PgPool) -> Result<Response, Rejection> {
    let notifications = vtstat_database::subscriptions::list(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&notifications).into_response())
}

async fn list_subscriptions(pool: PgPool) -> Result<Response, Rejection> {
    let subscriptions =
        vtstat_database::subscriptions::list_subscriptions::list_subscriptions(&pool)
            .await
            .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&subscriptions).into_response())
}

async fn list_vtubers(pool: PgPool) -> Result<Response, Rejection> {
    let vtubers = vtstat_database::vtubers::ListVtubersQuery {}
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&vtubers).into_response())
}

async fn re_run_job(job_id: i32, pool: PgPool) -> Result<Response, Rejection> {
    vtstat_database::jobs::re_run_job(job_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&()).into_response())
}

#[derive(Deserialize)]
pub struct CreateVTuberPayload {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub twitter_username: Option<String>,
    pub youtube_channel_id: String,
}

async fn create_vtuber(pool: PgPool, payload: CreateVTuberPayload) -> Result<Response, Rejection> {
    let mut channel = youtubei::browse_channel(&payload.youtube_channel_id)
        .await
        .map_err(WarpError)?;

    let mut thumbnail_url = channel
        .metadata
        .channel_metadata_renderer
        .avatar
        .thumbnails
        .pop()
        .map(|t| t.url);

    if let Some(url) = thumbnail_url.take() {
        async fn upload_thumbnail(url: &str, id: &str) -> anyhow::Result<String> {
            let client = Client::new();

            let file = client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            vtstat_utils::upload_file(&format!("thumbnail/{}.jpg", id), file, "image/jpg", &client)
                .await
        }

        if let Ok(url) = upload_thumbnail(&url, &payload.vtuber_id).await {
            thumbnail_url = Some(url);
        }
    }

    CreateVTuber {
        vtuber_id: payload.vtuber_id.clone(),
        native_name: payload.native_name,
        english_name: payload.english_name,
        japanese_name: payload.japanese_name,
        twitter_username: payload.twitter_username,
        thumbnail_url,
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    CreateChannel {
        platform: Platform::Youtube,
        platform_id: payload.youtube_channel_id,
        vtuber_id: payload.vtuber_id,
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::with_status(warp::reply::json(&()), StatusCode::CREATED).into_response())
}
