use sqlx::{PgPool, Result};

use super::Donation;

pub struct ListDonationsQuery {
    pub stream_id: i32,
    pub kind: &'static str,
}

impl ListDonationsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Donation>> {
        sqlx::query_as::<_, Donation>(
            r#"SELECT kind, time, value FROM donations WHERE stream_id = $1"#,
        )
        .bind(self.stream_id)
        .fetch_all(pool)
        .await
    }
}
