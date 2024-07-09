use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use tracing::Span;

use vtstats_database::channels::Platform;
use vtstats_database::streams::{
    get_stream_by_id, get_stream_by_platform_id, Column, Ordering, StreamStatus,
};

use crate::error::ApiResult;
use crate::AppContext;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListReqQuery {
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
pub struct FindByIdReqQuery {
    #[serde(default)]
    platform_id: Option<String>,
    #[serde(default)]
    platform: Option<Platform>,
    #[serde(default)]
    id: Option<i32>,
}

pub async fn find_stream_by_id(
    Query(query): Query<FindByIdReqQuery>,
    State(state): State<AppContext>,
) -> ApiResult<impl IntoResponse> {
    let stream = match (query.id, query.platform, query.platform_id) {
        (Some(id), None, None) => get_stream_by_id(id, &state.pool).await,
        (None, Some(platform), Some(platform_id)) => {
            get_stream_by_platform_id(platform, &platform_id, &state.pool).await
        }
        _ => return Ok(StatusCode::UNPROCESSABLE_ENTITY.into_response()),
    }?;

    if let Some(stream) = &stream {
        Span::current().record("stream_id", stream.stream_id);
    }

    Ok(Json(stream).into_response())
}

pub async fn list_scheduled_streams(
    Query(query): Query<ListReqQuery>,
    State(state): State<AppContext>,
) -> ApiResult<impl IntoResponse> {
    let streams = vtstats_database::streams::meilisearch::search(
        vtstats_database::streams::meilisearch::Search {
            channel_ids: query.channel_ids,
            status: StreamStatus::Scheduled,
            start_at: query.start_at,
            end_at: query.end_at,
            sort_by: Column::ScheduleTime,
            sort_direction: Ordering::Asc,
            ..Default::default()
        },
        &state.search,
    )
    .await?;

    Ok(Json(streams))
}

pub async fn list_live_streams(
    Query(query): Query<ListReqQuery>,
    State(state): State<AppContext>,
) -> ApiResult<impl IntoResponse> {
    let keyword = query
        .keyword
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let streams = vtstats_database::streams::meilisearch::search(
        vtstats_database::streams::meilisearch::Search {
            channel_ids: query.channel_ids,
            status: StreamStatus::Live,
            start_at: query.start_at,
            end_at: query.end_at,
            sort_by: Column::StartTime,
            sort_direction: Ordering::Desc,
            query: keyword,
            ..Default::default()
        },
        &state.search,
    )
    .await?;

    Ok(Json(streams))
}

pub async fn list_ended_streams(
    Query(query): Query<ListReqQuery>,
    State(state): State<AppContext>,
) -> ApiResult<impl IntoResponse> {
    let keyword = query
        .keyword
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let streams = vtstats_database::streams::meilisearch::search(
        vtstats_database::streams::meilisearch::Search {
            channel_ids: query.channel_ids,
            status: StreamStatus::Ended,
            start_at: query.start_at,
            end_at: query.end_at,
            sort_by: Column::StartTime,
            sort_direction: Ordering::Desc,
            query: keyword,
            ..Default::default()
        },
        &state.search,
    )
    .await?;

    Ok(Json(streams))
}
