use chrono::{DateTime, Utc};
use reqwest::Client;

use integration_bilibili::channels::channel_views;
use vtstats_database::{
    channel_stats_summary::{insert, AddChannelStats},
    channels::Channel,
    PgPool,
};

pub async fn run(
    channels: &[Channel],
    client: &Client,
    time: DateTime<Utc>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut view_stats = Vec::<(i32, i32)>::with_capacity(channels.len());

    for channel in channels {
        match channel_views(&channel.platform_id, client).await {
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
        insert(time, channel_id, AddChannelStats::View(value), pool).await?;
    }

    Ok(())
}
