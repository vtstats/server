use std::env;

use meilisearch_sdk::client::Client as Search;
use reqwest::{Client, ClientBuilder, Proxy};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct AppContext {
    pub pool: PgPool,
    pub search: Search,
    pub client: Client,

    pub database_url: String,
}

impl AppContext {
    pub async fn new() -> anyhow::Result<AppContext> {
        let database_url = env::var("DATABASE_URL")?;

        let pool = PgPoolOptions::new()
            .max_lifetime(std::time::Duration::from_secs(10 * 60)) // 10 minutes
            .connect(&database_url)
            .await?;

        let mut builder = ClientBuilder::new()
            .http1_only()
            .brotli(true)
            .deflate(true)
            .gzip(true);

        if let Ok(proxy) = env::var("ALL_PROXY") {
            builder = builder.proxy(Proxy::all(proxy)?);
        }

        let client = builder.build()?;

        let meilisearch_url = env::var("MEILI_URL")?;
        let meilisearch_master_key = env::var("MEILI_MASTER_KEY")?;

        let search = Search::new(meilisearch_url, Some(&meilisearch_master_key))?;

        Ok(AppContext {
            client,
            pool,
            search,
            database_url,
        })
    }
}
