use std::convert::Into;
use warp::Rejection;

use vtstat_database::{streams as db, PgPool};

use crate::reject::WarpError;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    vtuber_id: String,
}

pub async fn stream_times(query: ReqQuery, pool: PgPool) -> Result<impl warp::Reply, Rejection> {
    let times = db::stream_times(&query.vtuber_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&times))
}
