use std::convert::Into;
use warp::Rejection;

use vtstat_database::{streams as db, PgPool};

use crate::reject::WarpError;

#[derive(serde::Deserialize)]
pub struct ReqQuery {
    id: i32,
}

pub async fn stream_times(query: ReqQuery, pool: PgPool) -> Result<impl warp::Reply, Rejection> {
    tracing::info!("id={}", query.id);

    let times = db::stream_times(query.id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&times))
}
