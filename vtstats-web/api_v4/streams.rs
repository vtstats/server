use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use warp::{reply::Response, Rejection, Reply};

use vtstats_database::channels::Platform;
use vtstats_database::streams::{
    filter_streams_order_by_schedule_time_asc, filter_streams_order_by_start_time_desc,
    get_stream_by_platform_id, StreamStatus,
};
use vtstats_database::PgPool;

use crate::reject::WarpError;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    pub channel_ids: Vec<i32>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub end_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery_ {
    platform_id: String,
    platform: Platform,
}

pub async fn list_stream_by_platform_id(
    query: ReqQuery_,
    pool: PgPool,
) -> Result<Response, Rejection> {
    let streams = get_stream_by_platform_id(query.platform, &query.platform_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}

pub async fn list_scheduled_streams(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let streams = filter_streams_order_by_schedule_time_asc(
        &query.channel_ids,
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
        &query.channel_ids,
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
        &query.channel_ids,
        StreamStatus::Ended,
        query.start_at,
        query.end_at,
        pool,
    )
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}
