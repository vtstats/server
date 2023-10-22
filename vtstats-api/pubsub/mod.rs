pub mod publish;
pub mod verify;

use axum::{middleware, routing::post, Router};
use integration_youtube::pubsub::verify;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

pub use publish::publish_content;
pub use verify::verify_intent;
use vtstats_database::PgPool;

pub fn router(pool: PgPool) -> Router {
    let verify_layer = ServiceBuilder::new()
        .map_request_body(axum::body::boxed)
        .layer(middleware::from_fn(verify));

    Router::new()
        .route(
            "/",
            post(publish_content).layer(verify_layer).get(verify_intent),
        )
        .with_state(pool)
}
