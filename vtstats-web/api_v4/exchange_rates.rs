use warp::http::header;
use warp::reject::Rejection;
use warp::reply::{Reply, Response};

use vtstats_database::{exchange_rates::list_exchange_rates, PgPool};

use crate::reject::WarpError;

pub async fn exchange_rates(pool: PgPool) -> Result<Response, Rejection> {
    let res = list_exchange_rates(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::with_header(
        warp::reply::with_header(warp::reply::json(&res), header::VARY, "Origin"),
        header::CACHE_CONTROL,
        "max-age=864000", // 10 days
    )
    .into_response())
}
