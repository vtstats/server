use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use tracing::Span;
use warp::{reply::Response, Rejection, Reply};

use vtstats_database::channels::Platform;
use vtstats_database::streams::{
    filter_streams_order_by_schedule_time_asc, filter_streams_order_by_start_time_desc,
    get_stream_by_id, get_stream_by_platform_id, StreamStatus,
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
    #[serde(default)]
    pub keyword: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery_ {
    #[serde(default)]
    platform_id: Option<String>,
    #[serde(default)]
    platform: Option<Platform>,
    #[serde(default)]
    id: Option<i32>,
}

pub async fn list_stream_by_platform_id(
    query: ReqQuery_,
    pool: PgPool,
) -> Result<Response, Rejection> {
    let stream = match (query.id, query.platform, query.platform_id) {
        (Some(id), None, None) => get_stream_by_id(id, &pool).await,
        (None, Some(platform), Some(platform_id)) => {
            get_stream_by_platform_id(platform, &platform_id, &pool).await
        }
        _ => return Err(warp::reject::reject()),
    }
    .map_err(Into::<WarpError>::into)?;

    if let Some(stream) = &stream {
        Span::current().record("stream_id", stream.stream_id);
    }

    Ok(warp::reply::json(&stream).into_response())
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
        query
            .keyword
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty()),
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
        query
            .keyword
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty()),
        pool,
    )
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&streams).into_response())
}
