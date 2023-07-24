mod create_job;
mod create_vtuber;
mod re_run_job;

use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};
use warp::{reply::Response, Filter, Rejection, Reply};

use integration_googleauth::{validate, GoogleCerts};
use vtstat_database::{
    channels::ListChannelsQuery,
    streams::{Column, ListYouTubeStreamsQuery, Ordering},
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
        .and(warp::query())
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

    let create_job_api = warp::path!("admin" / "jobs")
        .and(warp::put())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and(warp::body::json())
        .and_then(create_job::create_job);

    let re_run_job_api = warp::path!("admin" / "jobs" / i32 / "re_run")
        .and(warp::post())
        .and(validate(certs.clone()))
        .and(with_pool(pool.clone()))
        .and_then(re_run_job::re_run_job);

    let create_vtuber_api = warp::path!("admin" / "vtuber")
        .and(warp::put())
        .and(validate(certs))
        .and(with_pool(pool))
        .and(warp::body::json())
        .and_then(create_vtuber::create_vtuber);

    jobs_api
        .or(streams_api)
        .or(notifications_api)
        .or(vtubers_api)
        .or(subscriptions_api)
        .or(channels_api)
        .or(create_job_api)
        .or(re_run_job_api)
        .or(create_vtuber_api)
}

#[derive(Deserialize)]
pub struct ListParameter {
    #[serde(default, with = "ts_milliseconds_option")]
    end_at: Option<DateTime<Utc>>,

    status: Option<String>,
}

async fn list_jobs(pool: PgPool, parameter: ListParameter) -> Result<Response, Rejection> {
    let jobs = vtstat_database::jobs::list_jobs_order_by_updated_at(
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
    let channels = ListChannelsQuery {
        platform: "youtube",
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&channels).into_response())
}

async fn list_notifications(pool: PgPool, parameter: ListParameter) -> Result<Response, Rejection> {
    let notifications = vtstat_database::subscriptions::list(parameter.end_at, &pool)
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

#[derive(Serialize)]
pub struct ActionResponse {
    msg: String,
}
