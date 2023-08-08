use serde::Serialize;
use sqlx::{PgPool, Result};

pub struct ListCurrenciesQuery;

#[derive(Serialize)]
pub struct Currency {
    pub code: String,
    pub rate: f64,
}

pub async fn list_currencies(pool: &PgPool) -> Result<Vec<Currency>> {
    let query = sqlx::query_as!(Currency, "SELECT code, rate FROM currencies").fetch_all(pool);

    crate::otel::instrument("SELECT", "currencies", query).await
}
