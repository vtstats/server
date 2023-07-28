pub mod publish;
pub mod verify;

use integration_youtube::pubsub::validate;

pub use publish::publish_content;
pub use verify::verify_intent;
use vtstat_database::PgPool;
use warp::{Filter, Rejection, Reply};

use crate::filters::with_pool;

pub fn routes(pool: PgPool) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_verify = warp::get().and(warp::query()).map(verify_intent);

    let api_publish = warp::post()
        .and(validate())
        .and(with_pool(pool))
        .and_then(publish_content);

    warp::path!("pubsub").and(api_verify.or(api_publish))
}
