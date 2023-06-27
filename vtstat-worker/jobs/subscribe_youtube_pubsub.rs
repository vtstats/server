use chrono::{Duration, DurationRound, Utc};
use futures::{stream, TryStreamExt};

use vtstat_database::channels::ListChannelsQuery;
use vtstat_database::PgPool;
use vtstat_request::RequestHub;

use super::JobResult;

pub async fn execute(pool: &PgPool, hub: RequestHub) -> anyhow::Result<JobResult> {
    let next_run = Utc::now().duration_trunc(Duration::days(1)).unwrap() + Duration::days(1);

    let channels = ListChannelsQuery {
        platform: "youtube",
    }
    .execute(pool)
    .await?;

    let _ = stream::unfold(channels.iter(), |mut iter| async {
        let channel = iter.next()?;
        let result = hub.subscribe_youtube_pubsub(&channel.platform_id).await;
        Some((result, iter))
    })
    .try_collect::<Vec<()>>()
    .await?;

    Ok(JobResult::Next {
        run: next_run,
        continuation: None,
    })
}
