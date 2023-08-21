mod types;

use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::Serialize;
use std::convert::Into;
use vtstats_database::{
    stream_events::{list_stream_events, StreamEventKind},
    PgPool,
};
use warp::Rejection;

use crate::reject::WarpError;

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
    pub value: RefinedStreamEventValue,
}

pub async fn stream_events(query: ReqQuery, pool: PgPool) -> Result<impl warp::Reply, Rejection> {
    let events = list_stream_events(query.stream_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

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

    Ok(warp::reply::json(&events))
}
