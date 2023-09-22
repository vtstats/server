use chrono::{DateTime, Utc};
use integration_bilibili::channels::channel_subscribers;
use reqwest::Client;

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
    let mut subscriber_stats = Vec::<(i32, i32)>::with_capacity(channels.len());

    for channel in channels {
        match channel_subscribers(&channel.platform_id, client).await {
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
        insert(time, channel_id, AddChannelStats::Subscriber(value), pool).await?;
    }

    Ok(())
}
