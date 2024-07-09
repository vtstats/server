use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub async fn channel_subscriber_stats(
    channel_id: i32,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<(i64, i32)>> {
    let query = sqlx::query!(
        r#"
 SELECT time ts, count v1
   FROM channel_subscriber_stats
  WHERE channel_id = $1
    AND (time >= $2 OR $2 IS NULL)
    AND (time <= $3 OR $3 IS NULL)
        "#,
        channel_id, // $1
        start_at,   // $2
        end_at,     // $3
    )
    .map(|row| (row.ts.timestamp_millis(), row.v1))
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_subscriber_stats", query)
}

pub async fn channel_subscriber_stats_at(
    channel_id: i32,
    at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Option<i32>> {
    let query = sqlx::query!(
        "SELECT count FROM channel_subscriber_stats WHERE channel_id = $1 AND time = $2",
        channel_id,
        at,
    )
    .fetch_optional(pool);

    let rec = crate::otel::execute_query!("SELECT", "channel_subscriber_stats", query)?;

    Ok(rec.map(|r| r.count))
}

pub async fn channel_subscriber_stats_insert(
    channel_id: i32,
    time: DateTime<Utc>,
    value: i32,
    pool: &PgPool,
) -> Result<()> {
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

    Ok(())
}
