use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgPool, Result};
use std::collections::HashMap;

pub enum AddChannelStats {
    View(i32),
    Subscriber(i32),
    Revenue(HashMap<String, Decimal>),
}

pub async fn insert(
    time: DateTime<Utc>,
    channel_id: i32,
    add: AddChannelStats,
    pool: &PgPool,
) -> Result<()> {
    match add {
        AddChannelStats::Subscriber(value) => {
            let query = sqlx::query!(
                "INSERT INTO channel_subscriber_stats as s (channel_id, time, count) \
                VALUES ($1, $2, $3) \
                ON CONFLICT (channel_id, time) DO UPDATE \
                SET count = GREATEST(excluded.count, s.count)",
                channel_id, // $1
                time,       // $2
                value,      // $3
            )
            .execute(pool);
            crate::otel::execute_query!("INSERT", "channel_subscriber_stats", query)?;

            let query = sqlx::query!(
                "UPDATE channel_stats_summary SET value = $1, updated_at = $6, \
                value_1_day_ago = COALESCE((SELECT to_jsonb(count) FROM channel_subscriber_stats WHERE time = $2 AND channel_id = $5), value_1_day_ago), \
                value_7_days_ago = COALESCE((SELECT to_jsonb(count) FROM channel_subscriber_stats WHERE time = $3 AND channel_id = $5), value_7_days_ago), \
                value_30_days_ago = COALESCE((SELECT to_jsonb(count) FROM channel_subscriber_stats WHERE time = $4 AND channel_id = $5), value_30_days_ago) \
                WHERE channel_id = $5 AND kind = 'subscriber'",
                json!(value), // $1
                time - Duration::days(1), // $2
                time - Duration::days(7), // $3
                time - Duration::days(30), // $4
                channel_id, // $5
                time, // $6
            )
            .execute(pool);
            crate::otel::execute_query!("UPDATE", "channel_stats_summary", query)?;
        }
        AddChannelStats::View(value) => {
            let query = sqlx::query!(
                "INSERT INTO channel_view_stats as s (channel_id, time, count) \
                VALUES ($1, $2, $3) \
                ON CONFLICT (channel_id, time) DO UPDATE \
                SET count = GREATEST(excluded.count, s.count)",
                channel_id, // $1
                time,       // $2
                value,      // $3
            )
            .execute(pool);
            crate::otel::execute_query!("INSERT", "channel_view_stats", query)?;

            let query = sqlx::query!(
                "UPDATE channel_stats_summary SET value = $1, updated_at = $6, \
                value_1_day_ago = COALESCE((SELECT to_jsonb(count) FROM channel_view_stats WHERE time = $2 AND channel_id = $5), value_1_day_ago), \
                value_7_days_ago = COALESCE((SELECT to_jsonb(count) FROM channel_view_stats WHERE time = $3 AND channel_id = $5), value_7_days_ago), \
                value_30_days_ago = COALESCE((SELECT to_jsonb(count) FROM channel_view_stats WHERE time = $4 AND channel_id = $5), value_30_days_ago) \
                WHERE channel_id = $5 AND kind = 'view'",
                json!(value), // $1
                time - Duration::days(1), // $2
                time - Duration::days(7), // $3
                time - Duration::days(30), // $4
                channel_id, // $5
                time, // $6
            )
            .execute(pool);
            crate::otel::execute_query!("UPDATE", "channel_stats_summary", query)?;
        }
        AddChannelStats::Revenue(value) => {
            let value = json!(value);
            let query = sqlx::query!(
                "INSERT INTO channel_revenue_stats(channel_id, time, value) \
                VALUES ($1, $2, $3) \
                ON CONFLICT (channel_id, time) DO UPDATE \
                SET value = excluded.value",
                channel_id, // $1
                time,       // $2
                &value,     // $3
            )
            .execute(pool);
            crate::otel::execute_query!("INSERT", "channel_revenue_stats", query)?;

            let query = sqlx::query!(
                "UPDATE channel_stats_summary SET value = $1, updated_at = $6, \
                value_1_day_ago = COALESCE((SELECT value FROM channel_revenue_stats WHERE time = $2 AND channel_id = $5), value_1_day_ago), \
                value_7_days_ago = COALESCE((SELECT value FROM channel_revenue_stats WHERE time = $3 AND channel_id = $5), value_7_days_ago), \
                value_30_days_ago = COALESCE((SELECT value FROM channel_revenue_stats WHERE time = $4 AND channel_id = $5), value_30_days_ago) \
                WHERE channel_id = $5 AND kind = 'revenue'",
                value, // $1
                time - Duration::days(1), // $2
                time - Duration::days(7), // $3
                time - Duration::days(30), // $4
                channel_id, // $5
                time, // $6
            )
            .execute(pool);
            crate::otel::execute_query!("UPDATE", "channel_stats_summary", query)?;
        }
    };

    Ok(())
}
