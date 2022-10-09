pub mod publish;
pub mod verify;

#[cfg(test)]
mod tests;

pub use publish::publish_content;
pub use verify::verify_intent;
use vtstat_database::PgPool;
use warp::Filter;

use crate::filters::{string_body, with_db};

pub fn pubsub(
    db: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("pubsub").and(pubsub_verify().or(pubsub_publish(db)))
}

pub fn pubsub_verify() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::query()).map(verify_intent)
}

pub fn pubsub_publish(
    db: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(string_body())
        .and(warp::header::<String>("x-hub-signature"))
        .and(with_db(db))
        .and_then(publish_content)
}
