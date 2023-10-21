pub mod publish;
pub mod verify;

use axum::{middleware, routing::get, Router};
use integration_youtube::pubsub::verify;

pub use publish::publish_content;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;
pub use verify::verify_intent;
use vtstats_database::PgPool;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(verify_intent).post(publish_content))
        .layer(
            ServiceBuilder::new()
                .map_request_body(axum::body::boxed)
                .layer(middleware::from_fn(verify)),
        )
        .with_state(pool)
}
