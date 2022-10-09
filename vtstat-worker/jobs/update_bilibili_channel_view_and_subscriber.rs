use futures::try_join;
use vtstat_database::{
    channel_stats::{
        AddChannelSubscriberStatsQuery, AddChannelSubscriberStatsRow, AddChannelViewStatsQuery,
        AddChannelViewStatsRow,
    },
    channels::ListChannelsQuery,
    PgPool,
};
use vtstat_request::RequestHub;

use super::JobResult;
use crate::timer::{timer, Calendar};

pub async fn execute(pool: PgPool, hub: RequestHub) -> anyhow::Result<JobResult> {
    let (current_run, next_run) = timer(Calendar::Hourly);

    let bilibili_channels = ListChannelsQuery {
        platform: "bilibili",
    }
    .execute(&pool)
    .await?;

    let mut channel_view_stats = Vec::with_capacity(bilibili_channels.len());
    let mut channel_subscribe_stats = Vec::with_capacity(bilibili_channels.len());

    for channel in bilibili_channels {
        let (stat, upstat) = try_join!(
            hub.bilibili_stat(&channel.platform_id),
            hub.bilibili_upstat(&channel.platform_id)
        )?;

        channel_view_stats.push(AddChannelViewStatsRow {
            channel_id: channel.channel_id,
            count: upstat.archive.view,
        });

        channel_subscribe_stats.push(AddChannelSubscriberStatsRow {
            channel_id: channel.channel_id,
            count: stat.follower,
        });
    }

    if !channel_view_stats.is_empty() {
        AddChannelViewStatsQuery {
            time: current_run,
            rows: channel_view_stats,
        }
        .execute(&pool)
        .await?;
    }

    if !channel_subscribe_stats.is_empty() {
        AddChannelSubscriberStatsQuery {
            time: current_run,
            rows: channel_subscribe_stats,
        }
        .execute(&pool)
        .await?;
    }

    Ok(JobResult::Next {
        run: next_run,
        continuation: None,
    })
}
