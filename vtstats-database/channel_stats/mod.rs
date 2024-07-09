mod meilisearch;
mod postgres;

pub use meilisearch::{list as channel_stats_summary, ChannelStatsKind};
pub use postgres::*;

use chrono::{DateTime, Duration, Utc};
use meilisearch_sdk::client::Client;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn channel_subscriber_stats_insert(
    time: DateTime<Utc>,
    channel_id: i32,
    value: i32,
    pool: &PgPool,
    client: &Client,
) -> anyhow::Result<()> {
    postgres::channel_subscriber_stats_insert(channel_id, time, value, pool).await?;

    let value_1_day_ago =
        postgres::channel_subscriber_stats_at(channel_id, time - Duration::days(1), pool).await?;
    let value_7_days_ago =
        postgres::channel_subscriber_stats_at(channel_id, time - Duration::days(7), pool).await?;
    let value_30_days_ago =
        postgres::channel_subscriber_stats_at(channel_id, time - Duration::days(30), pool).await?;

    meilisearch::add_or_update(
        meilisearch::Document {
            channel_id,
            updated_at: time,
            value: Some(json!(value)),
            value_1_day_ago: value_1_day_ago.map(|v| json!(v)),
            value_7_days_ago: value_7_days_ago.map(|v| json!(v)),
            value_30_days_ago: value_30_days_ago.map(|v| json!(v)),
        },
        ChannelStatsKind::Subscriber,
        client,
    )
    .await
}

pub async fn channel_view_stats_insert(
    time: DateTime<Utc>,
    channel_id: i32,
    value: i32,
    pool: &PgPool,
    client: &Client,
) -> anyhow::Result<()> {
    postgres::channel_view_stats_insert(channel_id, time, value, pool).await?;

    let value_1_day_ago =
        postgres::channel_view_stats_at(channel_id, time - Duration::days(1), pool).await?;
    let value_7_days_ago =
        postgres::channel_view_stats_at(channel_id, time - Duration::days(7), pool).await?;
    let value_30_days_ago =
        postgres::channel_view_stats_at(channel_id, time - Duration::days(30), pool).await?;

    meilisearch::add_or_update(
        meilisearch::Document {
            channel_id,
            updated_at: time,
            value: Some(json!(value)),
            value_1_day_ago: value_1_day_ago.map(|v| json!(v)),
            value_7_days_ago: value_7_days_ago.map(|v| json!(v)),
            value_30_days_ago: value_30_days_ago.map(|v| json!(v)),
        },
        ChannelStatsKind::View,
        client,
    )
    .await
}

pub async fn channel_revenue_stats_insert(
    time: DateTime<Utc>,
    channel_id: i32,
    value: HashMap<String, Decimal>,
    pool: &PgPool,
    client: &Client,
) -> anyhow::Result<()> {
    let value = json!(value);

    postgres::channel_revenue_stats_insert(channel_id, time, &value, pool).await?;

    let value_1_day_ago =
        postgres::channel_revenue_stats_at(channel_id, time - Duration::days(1), pool).await?;
    let value_7_days_ago =
        postgres::channel_revenue_stats_at(channel_id, time - Duration::days(7), pool).await?;
    let value_30_days_ago =
        postgres::channel_revenue_stats_at(channel_id, time - Duration::days(30), pool).await?;

    meilisearch::add_or_update(
        meilisearch::Document {
            channel_id,
            updated_at: time,
            value: Some(value),
            value_1_day_ago,
            value_7_days_ago,
            value_30_days_ago,
        },
        ChannelStatsKind::Revenue,
        client,
    )
    .await
}
