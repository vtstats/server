use futures_util::TryStreamExt;
use sqlx::{Either, PgPool, Result};
use std::collections::HashMap;

pub async fn list_exchange_rates(pool: &PgPool) -> Result<HashMap<String, f32>> {
    let query = sqlx::query!("SELECT code, rate FROM exchange_rates")
        .fetch_many(pool)
        .try_filter_map(|step| async move {
            Ok(match step {
                Either::Left(_) => None,
                Either::Right(o) => Some((o.code, o.rate)),
            })
        })
        .try_collect();

    crate::otel::execute_query!("SELECT", "exchange_rates", query)
}
