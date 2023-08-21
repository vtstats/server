use std::convert::Infallible;
use vtstats_database::PgPool;
use warp::Filter;

pub fn with_pool(pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
