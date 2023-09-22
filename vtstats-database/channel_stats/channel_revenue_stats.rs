use chrono::{DateTime, Utc};
use sqlx::{types::JsonValue, PgPool, Result};

pub async fn channel_revenue_stats(
    channel_id: i32,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: &PgPool,
) -> Result<Vec<(i64, JsonValue)>> {
    let query = sqlx::query!(
        r#"
 SELECT time ts, value v1
   FROM channel_revenue_stats
  WHERE channel_id = $1
    AND (time >= $2 OR $2 IS NULL)
    AND (time <= $3 OR $3 IS NULL)
        "#,
        channel_id, // $1
        start_at,   // $2
        end_at,     // $3
    )
    .map(|r| (r.ts.timestamp_millis(), r.v1))
    .fetch_all(pool);

    crate::otel::execute_query!("SELECT", "channel_revenue_stats", query)
}
