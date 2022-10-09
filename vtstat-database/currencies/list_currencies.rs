use sqlx::{PgPool, Result};

pub struct ListCurrenciesQuery;

pub struct Currency {
    pub code: String,
    pub symbol: String, 
}

impl ListCurrenciesQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Currency>> {
        let query = sqlx::query_as!(Currency, r#"SELECT code, symbol FROM currencies"#)
            .fetch_all(pool);

        crate::otel::instrument("SELECT", "currencies", query).await
    }
}

// TODO add unit tests
