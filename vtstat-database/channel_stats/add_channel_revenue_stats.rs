use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, Postgres, QueryBuilder, Result};

#[derive(Debug, Clone)]
pub struct ChannelRevenueStatsRow {
    pub channel_id: i32,
    pub value: HashMap<String, f32>,
}

pub async fn add_channel_revenue_stats(
    pool: &PgPool,
    time: DateTime<Utc>,
    rows: &[ChannelRevenueStatsRow],
) -> Result<()> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO channel_revenue_stats AS s (channel_id, time, value) ");

    query_builder.push_values(rows.iter(), |mut b, row| {
        b.push_bind(row.channel_id)
            .push_bind(time)
            .push_bind(Json(&row.value));
    });

    query_builder.push("ON CONFLICT (channel_id, time) DO UPDATE SET value = excluded.value");

    let query = query_builder.build().execute(pool);

    crate::otel::instrument("INSERT", "channel_revenue_stats", query).await?;

    Ok(())
}
