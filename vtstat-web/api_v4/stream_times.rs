use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use warp::Rejection;

use vtstat_database::{streams as db, PgPool};

use crate::reject::WarpError;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    channel_ids: Vec<i32>,
}

pub async fn stream_times(query: ReqQuery, pool: PgPool) -> Result<impl warp::Reply, Rejection> {
    let times = db::stream_times(&query.channel_ids, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&times))
}
