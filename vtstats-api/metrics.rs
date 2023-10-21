use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
use std::time::Instant;

pub async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let start = Instant::now();

    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    let method = req.method().clone();

    let response = next.run(req).await;

    metrics:: histogram!(
        "http_server_requests_elapsed_seconds",
        start.elapsed(),
        "method" => method.as_str().to_string(),
        "status_code" => response.status().as_str().to_string(),
        "path" => path.clone()
    );

    response
}
