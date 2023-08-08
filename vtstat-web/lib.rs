use metrics::{histogram, increment_counter};
use std::{env, net::SocketAddr};
use tokio::sync::oneshot::Receiver;
use tracing::{field::Empty, Span};
use vtstat_database::PgPool;
use warp::Filter;

// utils
mod filters;
mod reject;

// routes
mod api_admin;
mod api_discord;
mod api_pubsub;
mod api_sitemap;
mod api_v4;
// mod api_telegram;

pub async fn main(shutdown_rx: Receiver<()>) -> anyhow::Result<()> {
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let whoami = warp::path!("whoami").and(warp::get()).map(|| "OK");

    let routes = warp::path("api").and(
        whoami
            .or(api_sitemap::sitemap(pool.clone()))
            .or(api_v4::routes(pool.clone()))
            .or(api_discord::routes(pool.clone()))
            .or(api_admin::routes(pool.clone()))
            .or(api_pubsub::routes(pool)),
    );

    let filter = routes
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(&[
                    reqwest::Method::GET,
                    reqwest::Method::OPTIONS,
                    reqwest::Method::PUT,
                    reqwest::Method::POST,
                ])
                .allow_headers(&[
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::CONTENT_TYPE,
                ]),
        )
        .recover(reject::handle_rejection)
        .with(warp::log::custom(|info| {
            Span::current().record("http.res.status_code", info.status().as_u16());

            let method = info.method().as_str().to_string();
            let status_code = info.status().as_str().to_string();
            let path = info.path().to_string();

            histogram!(
                "http_server_requests_elapsed_seconds",
                info.elapsed(),
                "method" => method.clone(),
                "status_code" => status_code.clone(),
                "path" => path.clone()
            );
            increment_counter!(
                "http_server_requests_count",
                "method" => method,
                "status_code" => status_code,
                "path" => path
            );
        }))
        .with(warp::trace(|info| {
            if info.path() == "/api/whoami" {
                return tracing::trace_span!("Ignored");
            }

            let name = format!("{} {}", info.method().as_str(), info.path());

            let span = tracing::info_span!(
                "Http Server",
                "message" = name,
                "span.kind" = "server",
                "http.req.path" = info.path(),
                "http.req.method" = info.method().as_str(),
                "http.req.referer" = Empty,
                "http.res.status_code" = Empty,
            );

            if let Some(referer) = info.referer() {
                span.record("http.req.referer", referer);
            }

            span
        }));

    let address = env::var("SERVER_ADDRESS")?.parse::<SocketAddr>()?;

    tracing::warn!("Server started at {address}");

    let (_, server) = warp::serve(filter).bind_with_graceful_shutdown(address, async {
        shutdown_rx.await.ok();
    });

    server.await;

    tracing::warn!("Shutting down server...");

    Ok(())
}
