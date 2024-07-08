use sqlx::{PgPool, Result};
use std::collections::HashMap;

pub async fn list_exchange_rates(pool: &PgPool) -> Result<HashMap<String, f32>> {
    let query = sqlx::query!("SELECT code, rate FROM exchange_rates").fetch_all(pool);

    let vec = crate::otel::execute_query!("SELECT", "exchange_rates", query)?;

    Ok(vec.into_iter().map(|o| (o.code, o.rate)).collect())
}
