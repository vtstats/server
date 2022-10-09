use sqlx::{PgPool, Result};

use super::Channel;

pub struct ListChannelsQuery<'q> {
    pub platform: &'q str,
}

impl<'q> ListChannelsQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Channel>> {
        let query = sqlx::query_as!(
            Channel,
            r#"
     SELECT channel_id, platform_id, vtuber_id
       FROM channels
      WHERE platform::text = $1
            "#,
            self.platform
        )
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "channels", query).await
    }
}
