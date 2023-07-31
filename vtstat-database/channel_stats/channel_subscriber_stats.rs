use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Result};

use super::AddChannelSubscriberStatsRow;

#[derive(Serialize)]
pub struct ChannelsSubscriberStats {
    pub time: DateTime<Utc>,
    pub count: i32,
}

pub async fn channel_subscriber_stats(
    channel_id: i32,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<ChannelsSubscriberStats>> {
    let query = sqlx::query_as!(
        ChannelsSubscriberStats,
        r#"
 SELECT time, count
   FROM channel_subscriber_stats
  WHERE channel_id = $1
    AND (time >= $2 OR $2 IS NULL)
    AND (time <= $3 OR $3 IS NULL)
        "#,
        channel_id, // $1
        start_at,   // $2
        end_at,     // $3
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channel_subscriber_stats", query).await
}

pub async fn channel_subscriber_stats_before(
    before: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<AddChannelSubscriberStatsRow>> {
    let query = sqlx::query_as!(
        AddChannelSubscriberStatsRow,
        "SELECT channel_id, count FROM channel_subscriber_stats WHERE (time, channel_id) IN \
        (SELECT MAX(time), channel_id FROM channel_subscriber_stats WHERE time <= $1 GROUP BY channel_id)",
        before
    )
    .fetch_all(pool);

    crate::otel::instrument("SELECT", "channel_subscriber_stats", query).await
}

// TODO: add unit tests
