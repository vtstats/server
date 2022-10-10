use std::convert::Infallible;
use std::convert::Into;
use vtstat_database::PgPool;
use warp::Filter;

use crate::reject::WarpError;

pub fn with_pool(pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

pub fn string_body() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Copy {
    warp::body::bytes().and_then(|body: bytes::Bytes| async move {
        std::str::from_utf8(&body)
            .map(String::from)
            .map_err(Into::<WarpError>::into)
            .map_err(warp::reject::custom)
    })
}
