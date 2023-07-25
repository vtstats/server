pub mod otel;

pub mod channel_stats;
pub mod channels;
pub mod currencies;
pub mod donations;
pub mod jobs;
pub mod stream_stats;
pub mod streams;
pub mod subscriptions;
pub mod vtubers;

pub use sqlx::PgPool;

pub use sqlx::Error as DatabaseError;

pub use sqlx::postgres::PgListener;

pub async fn migrate() -> anyhow::Result<()> {
    let migrator = sqlx::migrate!();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    migrator.run(&pool).await?;

    Ok(())
}
