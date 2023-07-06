use std::{env, net::SocketAddr};
use tracing::field::Empty;
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
// mod api_telegram;

pub async fn main() -> anyhow::Result<()> {
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let whoami = warp::path!("whoami").and(warp::get()).map(|| "OK");

    let routes = warp::path("api").and(
        whoami
            .or(api_sitemap::sitemap(pool.clone()))
            // .or(api_telegram::routes(pool.clone()))
            .or(api_discord::routes(pool.clone()))
            .or(api_admin::routes(pool.clone()))
            .or(api_pubsub::verify())
            .or(api_pubsub::publish(pool)),
    );

    let filter = routes
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(&[
                    reqwest::Method::GET,
                    reqwest::Method::OPTIONS,
                    reqwest::Method::PUT,
                ])
                .allow_headers(&[
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::CONTENT_TYPE,
                ]),
        )
        .recover(reject::handle_rejection)
        .with(warp::trace(|info| {
            if info.path() == "/api/whoami" {
                return tracing::info_span!("Ignored");
            }

            let span = tracing::info_span!(
                "Request",
                name = Empty,
                span.kind = "server",
                service.name = "vtstat-web",
                req.path = info.path(),
                req.method = info.method().as_str(),
                req.referer = Empty,
                otel.status_code = Empty,
                otel.status_description = Empty,
                //// error
                error.message = Empty,
                error.cause_chain = Empty,
            );

            if let Some(referer) = info.referer() {
                span.record("req.referer", &referer);
            }

            span
        }));

    let address = env::var("SERVER_ADDRESS")?.parse::<SocketAddr>()?;

    println!("Server started at {address}");

    let (_, server) =
        warp::serve(filter).bind_with_graceful_shutdown(address, vtstat_utils::shutdown::signal());

    server.await;

    Ok(())
}
