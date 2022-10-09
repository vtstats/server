use filters::string_body;
use vtstat_database::PgPool;
use warp::Filter;

// utils
mod filters;
mod reject;

// routes
mod api_pubsub;
// mod api_sitemap;
// mod api_v4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vtstat_utils::dotenv::load();
    vtstat_utils::tracing::init("web");

    let pool = PgPool::connect("").await?;

    let with_pool = |pool: PgPool| warp::any().map(move || pool.clone());

    let verify_pubsub = warp::path!("/api/pubsub").and(
        warp::get()
            .and(warp::query())
            .map(api_pubsub::verify_intent),
    );

    let publish_content = warp::path!("/api/pubsub").and(
        warp::post()
            .and(string_body())
            .and(warp::header::<String>("x-hub-signature"))
            .and(with_pool(pool))
            .and_then(api_pubsub::publish_content),
    );

    let routes = verify_pubsub.or(publish_content);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
