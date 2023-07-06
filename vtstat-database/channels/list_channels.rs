use sqlx::{PgPool, Result};

use super::Channel;

pub struct ListChannelsQuery<'q> {
    pub platform: &'q str,
}

impl<'q> ListChannelsQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Channel>> {
        let query =
            sqlx::query_as::<_, Channel>(r#"SELECT * FROM channels WHERE platform::text = $1"#)
                .bind(self.platform)
                .fetch_all(pool);

        crate::otel::instrument("SELECT", "channels", query).await
    }
}
