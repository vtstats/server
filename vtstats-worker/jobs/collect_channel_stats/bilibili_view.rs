use chrono::{DateTime, Utc};
use integration_bilibili::channels::channel_views;
use vtstats_database::{channel_stats::channel_view_stats_insert, channels::Channel};
use vtstats_utils::context::AppContext;

pub async fn run(
    channels: &[Channel],
    time: DateTime<Utc>,
    ctx: &AppContext,
) -> anyhow::Result<()> {
    let mut view_stats = Vec::<(i32, i32)>::with_capacity(channels.len());

    for channel in channels {
        match channel_views(&channel.platform_id, &ctx.client).await {
            Ok(view) => view_stats.push((channel.channel_id, view)),
            Err(err) => {
                tracing::warn!(
                    "Failed to get channel stats vtuber_id={} platform=bilibili platform_id={}: {err}",
                    channel.vtuber_id,
                    channel.platform_id
                );
            }
        }
    }

    for (channel_id, value) in view_stats {
        channel_view_stats_insert(time, channel_id, value, &ctx.pool, &ctx.search).await?;
    }

    Ok(())
}
