use chrono::{DateTime, Utc};
use reqwest::Client;

use integration_youtube::data_api::channels::list_channels;
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

    let mut subscriber_stats = Vec::<(i32, i32)>::with_capacity(channels.len());

    // youtube limits 50 channels per request
    for chunk in channels.chunks(50) {
        let channel_ids = chunk.iter().fold(String::new(), |mut acc, channel| {
            if !acc.is_empty() {
                acc.push(',')
            }
            acc.push_str(&channel.platform_id);
            acc
        });

        let response = list_channels(&channel_ids, client).await?;

        for item in response.items {
            let channel = chunk.iter().find(|ch| ch.platform_id == item.id);

            let Some(channel) = channel else {
                tracing::warn!("Unktimen youtube channel id {}", item.id);
                continue;
            };

            let channel_id = channel.channel_id;

            if let Ok(value) = item.statistics.view_count.parse::<i32>() {
                view_stats.push((channel_id, value));
            }

            if let Ok(value) = item.statistics.subscriber_count.parse::<i32>() {
                subscriber_stats.push((channel_id, value));
            }
        }
    }

    // view stats
    for (channel_id, value) in view_stats {
        insert(time, channel_id, AddChannelStats::View(value), pool).await?;
    }

    for (channel_id, value) in subscriber_stats {
        insert(time, channel_id, AddChannelStats::Subscriber(value), pool).await?;
    }

    Ok(())
}
