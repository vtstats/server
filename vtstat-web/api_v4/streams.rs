use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use warp::{reply::Response, Rejection, Reply};

use vtstat_database::streams::{
    filter_streams_order_by_schedule_time_desc, filter_streams_order_by_start_time_desc,
    StreamStatus,
};
use vtstat_database::PgPool;

use crate::reject::WarpError;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub ids: Vec<String>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub end_at: Option<DateTime<Utc>>,
}

pub async fn list_scheduled_streams(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let streams = filter_streams_order_by_schedule_time_desc(
        &query.ids,
        StreamStatus::Scheduled,
        query.start_at,
        query.end_at,
        pool,
    )
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}

pub async fn list_live_streams(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let streams = filter_streams_order_by_start_time_desc(
        &query.ids,
        StreamStatus::Live,
        query.start_at,
        query.end_at,
        pool,
    )
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}

pub async fn list_ended_streams(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let streams = filter_streams_order_by_start_time_desc(
        &query.ids,
        StreamStatus::Ended,
        query.start_at,
        query.end_at,
        pool,
    )
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}
