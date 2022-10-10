use std::{env, net::SocketAddr};
use tracing::field::Empty;
use vtstat_database::PgPool;
use warp::Filter;

// utils
mod filters;
mod reject;

// routes
mod api_pubsub;
mod api_sitemap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vtstat_utils::dotenv::load();
    vtstat_utils::tracing::init("web");

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    vtstat_database::MIGRATOR.run(&pool).await?;

    let whoami = warp::path!("whoami").and(warp::get()).map(|| "OK");

    let routes = warp::path("api").and(
        whoami
            .or(api_sitemap::sitemap(pool.clone()))
            .or(api_pubsub::verify())
            .or(api_pubsub::publish(pool)),
    );

    let filter = routes
        .with(warp::cors().allow_any_origin())
        .recover(reject::handle_rejection)
        .with(warp::trace(|info| {
            let span = tracing::info_span!(
                "request",
                name = Empty,
                span.kind = "server",
                service.name = "vtstat-web",
                req.path = info.path(),
                req.method = info.method().as_str(),
                req.referer = Empty,
                otel.status_code = Empty,
                otel.status_description = Empty,
            );

            if let Some(referer) = info.referer() {
                span.record("req.referer", &referer);
            }

            span
        }));

    let address = env::var("SERVER_ADDRESS")?.parse::<SocketAddr>()?;

    println!("Server listening at {address}");

    warp::serve(filter).run(address).await;

    Ok(())
}
