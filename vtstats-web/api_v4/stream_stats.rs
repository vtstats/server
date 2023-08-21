use std::convert::Into;
use vtstats_database::{stream_stats as db, PgPool};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    stream_id: i32,
}

pub async fn stream_viewer_stats(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let stats = db::stream_viewer_stats(query.stream_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&stats).into_response())
}

pub async fn stream_chat_stats(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let stats = db::stream_chat_stats(query.stream_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&stats).into_response())
}
