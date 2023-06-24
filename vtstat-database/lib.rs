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

use sqlx::migrate::Migrator;

pub static MIGRATOR: Migrator = sqlx::migrate!();
