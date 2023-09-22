use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use warp::{reply::Response, Rejection, Reply};

use vtstats_database::{
    channel_stats_summary::{self, ChannelStatsKind},
    PgPool,
};

use crate::reject::WarpError;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    channel_ids: Vec<i32>,
    kind: ChannelStatsKind,
}

pub async fn channel_stats_summary(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let channels = channel_stats_summary::list(&query.channel_ids, query.kind, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&channels).into_response())
}
