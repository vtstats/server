mod create_job;
mod create_vtuber;
mod re_run_job;

use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};
use warp::{reply::Response, Filter, Rejection, Reply};

use integration_googleauth::{validate, GoogleCerts};
use vtstats_database::{
    streams::{Column, ListYouTubeStreamsQuery, Ordering},
    PgPool,
};

use crate::{filters::with_pool, reject::WarpError};

pub fn routes(pool: PgPool) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    let certs = GoogleCerts::new();

    let jobs_api = warp::path!("jobs")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::query())
        .and_then(list_jobs);

    let streams_api = warp::path!("streams")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::query())
        .and_then(list_streams);

    let notifications_api = warp::path!("notifications")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::query())
        .and_then(list_notifications);

    let vtubers_api = warp::path!("vtubers")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_vtubers);

    let subscriptions_api = warp::path!("subscriptions")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_subscriptions);

    let channels_api = warp::path!("channels")
        .and(warp::get())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(list_channels);

    let create_job_api = warp::path!("jobs")
        .and(warp::put())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::body::json())
        .and_then(create_job::create_job);

    let re_run_job_api = warp::path!("jobs" / i32 / "re_run")
        .and(warp::post())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(re_run_job::re_run_job);

    let create_vtuber_api = warp::path!("vtuber")
        .and(warp::put())
        .and(validate(certs))
        .and(with_pool(pool))
        .and(warp::body::json())
        .and_then(create_vtuber::create_vtuber);

    warp::path("admin").and(
        jobs_api
            .or(streams_api)
            .or(notifications_api)
            .or(vtubers_api)
            .or(subscriptions_api)
            .or(channels_api)
            .or(create_job_api)
            .or(re_run_job_api)
            .or(create_vtuber_api),
    )
}

#[derive(Deserialize)]
pub struct ListParameter {
    #[serde(default, with = "ts_milliseconds_option")]
    end_at: Option<DateTime<Utc>>,

    status: Option<String>,
}

async fn list_jobs(pool: PgPool, parameter: ListParameter) -> Result<Response, Rejection> {
    let jobs = vtstats_database::jobs::list_jobs_order_by_updated_at(
        parameter.status.unwrap_or_else(|| "queued".into()),
        parameter.end_at,
        &pool,
    )
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&jobs).into_response())
}

async fn list_streams(pool: PgPool, parameter: ListParameter) -> Result<Response, Rejection> {
    let status = match parameter.status.as_deref() {
        Some("scheduled") => "scheduled",
        Some("live") => "live",
        _ => "ended",
    };

    let streams = ListYouTubeStreamsQuery {
        limit: Some(24),
        order_by: Some((Column::UpdatedAt, Ordering::Desc)),
        end_at: parameter.end_at.as_ref().map(|dt| (Column::UpdatedAt, dt)),
        status: &[status.into()],
        ..Default::default()
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}

async fn list_channels(pool: PgPool) -> Result<Response, Rejection> {
    let channels = vtstats_database::channels::list_youtube_channels(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&channels).into_response())
}

async fn list_notifications(pool: PgPool, parameter: ListParameter) -> Result<Response, Rejection> {
    let notifications = vtstats_database::subscriptions::list(parameter.end_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&notifications).into_response())
}

async fn list_subscriptions(pool: PgPool) -> Result<Response, Rejection> {
    let subscriptions =
        vtstats_database::subscriptions::list_subscriptions::list_subscriptions(&pool)
            .await
            .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&subscriptions).into_response())
}

async fn list_vtubers(pool: PgPool) -> Result<Response, Rejection> {
    let vtubers = vtstats_database::vtubers::ListVtubersQuery {}
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&vtubers).into_response())
}

#[derive(Serialize)]
pub struct ActionResponse {
    msg: String,
}
