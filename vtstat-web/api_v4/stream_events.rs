use std::convert::Into;
use vtstat_database::{stream_events::list_stream_events, PgPool};
use warp::Rejection;

use crate::reject::WarpError;

#[derive(serde::Deserialize)]
pub struct ReqQuery {
    id: i32,
}

pub async fn stream_events(query: ReqQuery, pool: PgPool) -> Result<impl warp::Reply, Rejection> {
    let events = list_stream_events(query.id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&events))
}
