use chrono::{DateTime, Utc};
use integration_bilibili::channels::channel_subscribers;
use vtstats_database::{channel_stats::channel_subscriber_stats_insert, channels::Channel};
use vtstats_utils::context::AppContext;

pub async fn run(
    channels: &[Channel],
    time: DateTime<Utc>,
    ctx: &AppContext,
) -> anyhow::Result<()> {
    let mut subscriber_stats = Vec::<(i32, i32)>::with_capacity(channels.len());

    for channel in channels {
        match channel_subscribers(&channel.platform_id, &ctx.client).await {
            Ok(subscribers) => subscriber_stats.push((channel.channel_id, subscribers)),
            Err(err) => {
                tracing::warn!(
                    "Failed to get channel stats vtuber_id={} platform=bilibili platform_id={}: {err}",
                    channel.vtuber_id,
                    channel.platform_id
                );
            }
        }
    }

    for (channel_id, value) in subscriber_stats {
        channel_subscriber_stats_insert(time, channel_id, value, &ctx.pool, &ctx.search).await?;
    }

    Ok(())
}
