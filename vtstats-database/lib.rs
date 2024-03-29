#![warn(clippy::print_stdout)]
#![warn(clippy::todo)]
#![warn(clippy::unwrap_used)]

pub mod json;
pub mod otel;

pub mod channel_stats;
pub mod channel_stats_summary;
pub mod channels;
pub mod exchange_rates;
pub mod groups;
pub mod jobs;
pub mod stream_events;
pub mod stream_stats;
pub mod streams;
pub mod subscriptions;
pub mod vtubers;

pub use sqlx::PgPool;

pub use sqlx::Error as DatabaseError;

pub use sqlx::postgres::PgListener;
pub use sqlx::postgres::PgPoolOptions;

pub async fn migrate() -> anyhow::Result<()> {
    let migrator = sqlx::migrate!();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    migrator.run(&pool).await?;

    Ok(())
}
