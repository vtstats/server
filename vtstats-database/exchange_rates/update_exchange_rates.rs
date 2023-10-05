use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Result};

pub async fn update_exchange_rates(
    pool: &PgPool,
    time: DateTime<Utc>,
    iter: impl Iterator<Item = (String, f32)>,
) -> Result<()> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO exchange_rates (code, rate, updated_at) ");

    query_builder.push_values(iter, |mut b, row| {
        b.push_bind(row.0).push_bind(row.1).push_bind(time);
    });

    query_builder.push(
        "ON CONFLICT (code) DO UPDATE SET rate = excluded.rate, updated_at = excluded.updated_at",
    );

    let query = query_builder.build().execute(pool);

    crate::otel::execute_query!("INSERT", "exchange_rates", query)?;

    Ok(())
}
