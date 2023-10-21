mod types;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::Serialize;
use tracing::Span;
use vtstats_database::{
    stream_events::{list_stream_events, StreamEventKind},
    PgPool,
};

use crate::error::ApiResult;

use self::types::{refine, RefinedStreamEventValue};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    stream_id: i32,
}

#[derive(Debug, Serialize)]
pub struct StreamEvent {
    #[serde(with = "ts_milliseconds")]
    pub time: DateTime<Utc>,
    pub kind: StreamEventKind,
    #[serde(skip_serializing_if = "RefinedStreamEventValue::is_empty")]
    pub value: RefinedStreamEventValue,
}

pub async fn stream_events(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<impl IntoResponse> {
    let events = list_stream_events(query.stream_id, &pool).await?;

    let events: Vec<_> = events
        .into_iter()
        .filter_map(|event| {
            Some(StreamEvent {
                time: event.time,
                kind: event.kind,
                value: refine(event.value)?,
            })
        })
        .collect();

    Span::current().record("stream_id", query.stream_id);

    Ok(Json(events))
}
