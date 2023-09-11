use chrono::{Duration, DurationRound, Utc};
use integration_twitch::gql::use_view_count;
use reqwest::Client;
use vtstats_database::{stream_stats::AddStreamViewerStatsQuery, PgPool};

pub async fn collect_viewers(
    stream_id: i32,
    login: &str,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    loop {
        let res = use_view_count(login.to_string(), client).await?;

        if let Some(stream) = res.data.user.stream {
            AddStreamViewerStatsQuery {
                stream_id,
                time: Utc::now().duration_trunc(Duration::seconds(15)).unwrap(),
                count: stream.viewers_count,
            }
            .execute(pool)
            .await?;
        }

        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}
