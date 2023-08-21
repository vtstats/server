use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use vtstats_database::{channels as db, PgPool};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    channel_ids: Vec<i32>,
}

pub async fn channel_stats_summary(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let channels = db::list_channels_with_stats(&query.channel_ids, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&channels).into_response())
}
