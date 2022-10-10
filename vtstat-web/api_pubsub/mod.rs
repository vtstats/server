pub mod publish;
pub mod verify;

#[cfg(test)]
mod tests;

pub use publish::publish_content;
pub use verify::verify_intent;
use vtstat_database::PgPool;
use warp::Filter;

use crate::filters::{string_body, with_pool};

pub fn verify() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("pubsub")
        .and(warp::get())
        .and(warp::query())
        .map(verify_intent)
}

pub fn publish(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("pubsub")
        .and(warp::post())
        .and(string_body())
        .and(warp::header::<String>("x-hub-signature"))
        .and(with_pool(pool))
        .and_then(publish_content)
}
