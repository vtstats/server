use chrono::{DateTime, Utc};
use futures::TryFutureExt;
use reqwest::Client;

use integration_twitch::gql::{channel_avatar, channel_panels};
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
        match channel_panels(&channel.platform_id, client)
            .and_then(|res| channel_avatar(res.data.user.login, client))
            .await
        {
            Ok(res) => {
                subscriber_stats.push((channel.channel_id, res.data.user.followers.total_count))
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to get channel stats vtuber_id={} platform=twitch platform_id={}: {err}",
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
