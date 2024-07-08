use axum::{
    extract::{MatchedPath, Request},
    http::{
        header::{self, REFERER},
        Method,
    },
    middleware::{from_fn, Next},
    response::{IntoResponse, Response},
    Router,
};
use std::{env, net::SocketAddr, time::Duration, time::Instant};
use tokio::sync::oneshot::Receiver;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{field::Empty, Span};
use vtstats_utils::context::AppContext;

// utils
mod error;

// routes
mod admin;
mod discord;
mod pubsub;
mod sitemap;
// mod telegram;
mod twitch;
mod v4;

pub async fn main(shutdown_rx: Receiver<()>) -> anyhow::Result<()> {
    let address = env::var("SERVER_ADDRESS")?.parse::<SocketAddr>()?;

    let state = AppContext::new().await?;

    let app = Router::new()
        .nest("/api/v4", v4::router(state.clone()))
        .nest("/api/admin", admin::router(state.clone()))
        .nest("/api/discord", discord::router(state.clone()))
        .nest("/api/pubsub", pubsub::router(state.clone()))
        .nest("/api/sitemap", sitemap::router(state.clone()))
        .nest("/api/twitch", twitch::router(state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or("/404");

                    let referer = req
                        .headers()
                        .get(REFERER)
                        .map(|v| v.to_str().ok())
                        .unwrap_or_default();

                    let name = format!("{} {}", req.method().as_str(), matched_path);

                    tracing::info_span!(
                        "Http Server",
                        "message" = name,
                        "span.kind" = "server",
                        "http.req.path" = matched_path,
                        "http.req.method" = req.method().as_str(),
                        "http.req.referer" = referer,
                        "http.res.status_code" = Empty,
                    )
                })
                .on_response(|response: &Response, _latency: Duration, span: &Span| {
                    span.record("http.res.status_code", response.status().as_str());
                }),
        )
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::OPTIONS, Method::PUT, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_origin(AllowOrigin::predicate(|value, _| {
                    value.as_bytes().ends_with(b".poi.cat")
                        || value.as_bytes().ends_with(b".vtstats.pages.dev")
                        || value.as_bytes().ends_with(b"/vtstats.pages.dev")
                        || value.as_bytes().ends_with(b"/localhost:4200")
                })),
        )
        .layer(from_fn(track_metrics));

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    tracing::warn!(
        "API server is listening on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await?;

    tracing::warn!("API server is shutting down...");

    Ok(())
}

async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let Some(matched_path) = req.extensions().get::<MatchedPath>() else {
        return next.run(req).await;
    };

    let start = Instant::now();
    let path = matched_path.as_str().to_owned();
    let method = req.method().clone();

    let response = next.run(req).await;

    metrics::histogram!(
        "http_server_requests_elapsed_seconds",
        "method" => method.as_str().to_string(),
        "status_code" => response.status().as_str().to_string(),
        "path" => path.clone()
    )
    .record(start.elapsed());

    response
}
