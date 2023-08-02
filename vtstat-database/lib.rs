pub mod otel;

pub mod channel_stats;
pub mod channels;
pub mod currencies;
pub mod jobs;
pub mod stream_events;
pub mod stream_stats;
pub mod streams;
pub mod subscriptions;
pub mod vtubers;

use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};
pub use sqlx::PgPool;

pub use sqlx::Error as DatabaseError;

pub use sqlx::postgres::PgListener;

pub async fn migrate() -> anyhow::Result<()> {
    let migrator = sqlx::migrate!();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    migrator.run(&pool).await?;

    Ok(())
}

pub struct SeriesData {
    pub ts: DateTime<Utc>,
    pub v1: i32,
}

impl SeriesData {
    pub fn new(ts: DateTime<Utc>, v1: i32) -> Self {
        SeriesData { ts, v1 }
    }
}

impl SeriesData2 {
    pub fn new(ts: DateTime<Utc>, v1: i32, v2: i32) -> Self {
        SeriesData2 { ts, v1, v2 }
    }
}

pub struct SeriesData2 {
    pub ts: DateTime<Utc>,
    pub v1: i32,
    pub v2: i32,
}

impl Serialize for SeriesData {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.ts.timestamp_millis(), self.v1).serialize(s)
    }
}

impl Serialize for SeriesData2 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.ts.timestamp_millis(), self.v1, self.v2).serialize(s)
    }
}
