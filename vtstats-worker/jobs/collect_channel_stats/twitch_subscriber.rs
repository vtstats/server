use chrono::{DateTime, Utc};
use futures::TryFutureExt;
use integration_twitch::gql::{channel_avatar, channel_panels};
use vtstats_database::{channel_stats::channel_subscriber_stats_insert, channels::Channel};
use vtstats_utils::context::AppContext;

pub async fn run(
    channels: &[Channel],
    time: DateTime<Utc>,
    ctx: &AppContext,
) -> anyhow::Result<()> {
    let mut subscriber_stats = Vec::<(i32, i32)>::with_capacity(channels.len());

    for channel in channels {
        match channel_panels(&channel.platform_id, &ctx.client)
            .and_then(|res| channel_avatar(res.data.user.login, &ctx.client))
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
        channel_subscriber_stats_insert(time, channel_id, value, &ctx.pool, &ctx.search).await?;
    }

    Ok(())
}
