pub mod publish;
pub mod verify;

use axum::{middleware, routing::post, Router};
use integration_youtube::pubsub::verify;

pub use publish::publish_content;
pub use verify::verify_intent;

use crate::AppContext;

pub fn router(state: AppContext) -> Router {
    Router::new()
        .route(
            "/",
            post(publish_content)
                .layer(middleware::from_fn(verify))
                .get(verify_intent),
        )
        .with_state(state)
}
