use axum::{
    extract::MatchedPath,
    http::{header, header::REFERER, Method, Request},
    middleware::{from_fn, Next},
    response::{IntoResponse, Response},
    Router,
};
use std::{env, net::SocketAddr, time::Duration, time::Instant};
use tokio::sync::oneshot::Receiver;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{field::Empty, Span};
use vtstats_database::PgPoolOptions;

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
    let pool = PgPoolOptions::new()
        .max_lifetime(Duration::from_secs(10 * 60)) // 10 minutes
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    let address = env::var("SERVER_ADDRESS")?.parse::<SocketAddr>()?;

    let app = Router::new()
        .nest("/api/v4", v4::router(pool.clone()))
        .nest("/api/admin", admin::router(pool.clone()))
        .nest("/api/discord", discord::router(pool.clone()))
        .nest("/api/pubsub", pubsub::router(pool.clone()))
        .nest("/api/sitemap", sitemap::router(pool.clone()))
        .nest("/api/twitch", twitch::router(pool.clone()));

    let layers = ServiceBuilder::new()
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

    tracing::warn!("API server is listening on {address}");

    axum::Server::bind(&address)
        .serve(app.layer(layers).into_make_service())
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await?;

    tracing::warn!("API server is shutting down...");

    Ok(())
}

async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let Some(matched_path) = req.extensions().get::<MatchedPath>() else {
        return next.run(req).await;
    };

    let start = Instant::now();
    let path = matched_path.as_str().to_owned();
    let method = req.method().clone();

    let response = next.run(req).await;

    metrics::histogram!(
        "http_server_requests_elapsed_seconds",
        start.elapsed(),
        "method" => method.as_str().to_string(),
        "status_code" => response.status().as_str().to_string(),
        "path" => path.clone()
    );

    response
}
