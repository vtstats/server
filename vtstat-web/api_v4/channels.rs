use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use std::convert::Into;
use vtstat_database::{channels as db, channels::Platform, PgPool};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[serde_as]
#[derive(serde::Deserialize)]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    ids: Vec<String>,
    platform: Platform,
}

pub async fn list_channels(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let channels = db::list_channels(&query.ids, query.platform, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&channels).into_response())
}
