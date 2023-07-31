use chrono::{Duration, DurationRound, Utc};
use integration_youtube::data_api::channels::list_channels;
use reqwest::Client;
use tokio::try_join;
use vtstat_database::{
    channel_stats::{
        channel_subscriber_stats_before, channel_view_stats_before, AddChannelSubscriberStatsQuery,
        AddChannelSubscriberStatsRow, AddChannelViewStatsQuery, AddChannelViewStatsRow,
    },
    channels::{list_youtube_channels, update_channel_stats, Channel, ChannelStatsSummary},
    PgPool,
};
use vtstat_request::RequestHub;

use super::JobResult;

pub async fn execute(pool: &PgPool, hub: RequestHub) -> anyhow::Result<JobResult> {
    let now = Utc::now().duration_trunc(Duration::hours(1)).unwrap();

    let youtube_channels = list_youtube_channels(pool).await?;

    let (ref mut view_stats, ref mut subscriber_stats) =
        youtube_channels_stats(&youtube_channels, &hub.client).await?;

    let (
        ref mut view_stats_1d_ago,
        ref mut view_stats_7d_ago,
        ref mut view_stats_30d_ago,
        ref mut subscriber_stats_1d_ago,
        ref mut subscriber_stats_7d_ago,
        ref mut subscriber_stats_30d_ago,
    ) = try_join!(
        channel_view_stats_before(now - Duration::days(1), pool),
        channel_view_stats_before(now - Duration::days(7), pool),
        channel_view_stats_before(now - Duration::days(30), pool),
        channel_subscriber_stats_before(now - Duration::days(1), pool),
        channel_subscriber_stats_before(now - Duration::days(7), pool),
        channel_subscriber_stats_before(now - Duration::days(30), pool),
    )?;

    if !view_stats.is_empty() {
        AddChannelViewStatsQuery {
            time: now,
            rows: &view_stats,
        }
        .execute(pool)
        .await?;
    }

    if !subscriber_stats.is_empty() {
        AddChannelSubscriberStatsQuery {
            time: now,
            rows: &subscriber_stats,
        }
        .execute(pool)
        .await?;
    }

    macro_rules! find_map {
        ($vec:ident, $id:expr) => {
            $vec.iter()
                .position(|v| v.channel_id == $id)
                .map(|i| $vec.swap_remove(i).count)
        };
    }

    let rows = youtube_channels.into_iter().map(|c| ChannelStatsSummary {
        channel_id: c.channel_id,
        view: find_map!(view_stats, c.channel_id),
        view_1d_ago: find_map!(view_stats_1d_ago, c.channel_id),
        view_7d_ago: find_map!(view_stats_7d_ago, c.channel_id),
        view_30d_ago: find_map!(view_stats_30d_ago, c.channel_id),
        subscriber: find_map!(subscriber_stats, c.channel_id),
        subscriber_1d_ago: find_map!(subscriber_stats_1d_ago, c.channel_id),
        subscriber_7d_ago: find_map!(subscriber_stats_7d_ago, c.channel_id),
        subscriber_30d_ago: find_map!(subscriber_stats_30d_ago, c.channel_id),
    });

    update_channel_stats(rows, pool).await?;

    Ok(JobResult::Next {
        run: now + Duration::hours(1),
        continuation: None,
    })
}

async fn youtube_channels_stats(
    channels: &[Channel],
    client: &Client,
) -> anyhow::Result<(
    Vec<AddChannelViewStatsRow>,
    Vec<AddChannelSubscriberStatsRow>,
)> {
    let mut view_stats = Vec::with_capacity(channels.len());
    let mut subscribe_stats = Vec::with_capacity(channels.len());

    // youtube limits 50 channels per request
    for chunk in channels.chunks(50) {
        let channel_ids = chunk.iter().fold(String::new(), |mut acc, channel| {
            if !acc.is_empty() {
                acc.push(',')
            }
            acc.push_str(&channel.platform_id);
            acc
        });

        let response = list_channels(&channel_ids, &client).await?;

        for item in response.items {
            let channel = chunk.iter().find(|ch| ch.platform_id == item.id);

            let Some(channel) = channel else {
                tracing::warn!("Unknown youtube channel id {}", item.id);
                continue;
            };

            view_stats.push(AddChannelViewStatsRow {
                channel_id: channel.channel_id,
                count: item.statistics.view_count.parse().unwrap_or_default(),
            });

            subscribe_stats.push(AddChannelSubscriberStatsRow {
                channel_id: channel.channel_id,
                count: item.statistics.subscriber_count.parse().unwrap_or_default(),
            });
        }
    }

    Ok((view_stats, subscribe_stats))
}
