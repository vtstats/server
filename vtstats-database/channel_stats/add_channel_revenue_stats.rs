use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{types::Json, PgPool, Postgres, QueryBuilder, Result};
use std::collections::HashMap;

pub async fn add_channel_revenue_stats(
    pool: &PgPool,
    rows: &[(i32, DateTime<Utc>, HashMap<String, Decimal>)],
) -> Result<()> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO channel_revenue_stats AS s (channel_id, time, value) ");

    query_builder.push_values(rows.iter(), |mut b, row| {
        b.push_bind(row.0).push_bind(row.1).push_bind(Json(&row.2));
    });

    query_builder.push("ON CONFLICT (channel_id, time) DO UPDATE SET value = excluded.value");

    let query = query_builder.build().execute(pool);

    crate::otel::execute_query!("INSERT", "channel_revenue_stats", query)?;

    Ok(())
}
