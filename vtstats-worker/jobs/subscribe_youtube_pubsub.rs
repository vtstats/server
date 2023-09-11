use chrono::{Duration, DurationRound, Utc};
use futures::{stream, TryStreamExt};
use reqwest::Client;

use integration_youtube::pubsub::SubscribeYouTubePubsubQuery;
use vtstats_database::channels::{list_active_channels_by_platform, Platform};
use vtstats_database::PgPool;

use super::JobResult;

pub async fn execute(pool: &PgPool, client: Client) -> anyhow::Result<JobResult> {
    let next_run = Utc::now().duration_trunc(Duration::days(1)).unwrap() + Duration::days(1);

    let channels = list_active_channels_by_platform(Platform::Youtube, pool).await?;

    let _ = stream::unfold(channels.iter(), |mut iter| async {
        let channel = iter.next()?;
        let result = SubscribeYouTubePubsubQuery {
            channel_id: channel.platform_id.to_string(),
        }
        .send(&client)
        .await;
        Some((result, iter))
    })
    .try_collect::<Vec<()>>()
    .await?;

    Ok(JobResult::Next { run: next_run })
}
