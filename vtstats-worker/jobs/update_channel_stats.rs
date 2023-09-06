use chrono::{DateTime, Duration, DurationRound, Utc};
use reqwest::Client;
use tokio::try_join;
use vtstats_database::{
    channel_stats::{
        add_channel_revenue_stats, channel_revenue_stats_before, channel_subscriber_stats_before,
        channel_view_stats_before, AddChannelSubscriberStatsQuery, AddChannelSubscriberStatsRow,
        AddChannelViewStatsQuery, AddChannelViewStatsRow, ChannelRevenueStatsRow,
    },
    channels::{
        list_bilibili_channels, list_youtube_channels, update_channel_stats, Channel,
        ChannelStatsSummary,
    },
    stream_events::list_revenue_by_channel_start_at,
    PgPool,
};
use vtstats_utils::currency::currency_symbol_to_code;

use super::JobResult;

pub async fn execute(pool: &PgPool, client: &Client) -> anyhow::Result<JobResult> {
    let now = Utc::now().duration_trunc(Duration::hours(1)).unwrap();

    let youtube_channels = list_youtube_channels(pool).await?;
    let bilibili_channels = list_bilibili_channels(pool).await?;

    let (mut youtube_view_stats, mut youtube_subscriber_stats) =
        youtube_channels_stats(&youtube_channels, client)
            .await
            .unwrap_or_else(|err| {
                tracing::error!(exception.stacktrace = ?err, message= %err);
                (vec![], vec![])
            });

    let mut youtube_revenue_stats =
        channel_revenue_stats(&youtube_channels, now - Duration::hours(1), pool)
            .await
            .unwrap_or_else(|err| {
                tracing::error!(exception.stacktrace = ?err, message= %err);
                vec![]
            });

    let (mut bilibili_view_stats, mut bilibili_subscriber_stats) =
        bilibili_channels_stats(&bilibili_channels, client)
            .await
            .unwrap_or_else(|err| {
                tracing::error!(exception.stacktrace = ?err, message= %err);
                (vec![], vec![])
            });

    let view_stats = &mut youtube_view_stats;
    view_stats.append(&mut bilibili_view_stats);
    let subscriber_stats = &mut youtube_subscriber_stats;
    subscriber_stats.append(&mut bilibili_subscriber_stats);
    let revenue_stats = &mut youtube_revenue_stats;

    let (
        ref mut view_stats_1d_ago,
        ref mut view_stats_7d_ago,
        ref mut view_stats_30d_ago,
        ref mut subscriber_stats_1d_ago,
        ref mut subscriber_stats_7d_ago,
        ref mut subscriber_stats_30d_ago,
        ref mut revenue_stats_1d_ago,
        ref mut revenue_stats_7d_ago,
        ref mut revenue_stats_30d_ago,
    ) = try_join!(
        channel_view_stats_before(now - Duration::days(1), pool),
        channel_view_stats_before(now - Duration::days(7), pool),
        channel_view_stats_before(now - Duration::days(30), pool),
        channel_subscriber_stats_before(now - Duration::days(1), pool),
        channel_subscriber_stats_before(now - Duration::days(7), pool),
        channel_subscriber_stats_before(now - Duration::days(30), pool),
        channel_revenue_stats_before(now - Duration::days(1), pool),
        channel_revenue_stats_before(now - Duration::days(7), pool),
        channel_revenue_stats_before(now - Duration::days(30), pool),
    )?;

    if !revenue_stats.is_empty() {
        add_channel_revenue_stats(pool, now, revenue_stats).await?;
    }

    if !view_stats.is_empty() {
        AddChannelViewStatsQuery {
            time: now,
            rows: view_stats,
        }
        .execute(pool)
        .await?;
    }

    if !subscriber_stats.is_empty() {
        AddChannelSubscriberStatsQuery {
            time: now,
            rows: subscriber_stats,
        }
        .execute(pool)
        .await?;
    }

    macro_rules! find_map {
        ($vec:ident, $id:expr) => {
            $vec.iter()
                .position(|v| v.channel_id == $id)
                .map(|i| $vec.swap_remove(i).value)
        };
    }

    let rows = youtube_channels
        .into_iter()
        .chain(bilibili_channels.into_iter())
        .map(|c| ChannelStatsSummary {
            channel_id: c.channel_id,
            view: find_map!(view_stats, c.channel_id),
            view_1d_ago: find_map!(view_stats_1d_ago, c.channel_id),
            view_7d_ago: find_map!(view_stats_7d_ago, c.channel_id),
            view_30d_ago: find_map!(view_stats_30d_ago, c.channel_id),
            subscriber: find_map!(subscriber_stats, c.channel_id),
            subscriber_1d_ago: find_map!(subscriber_stats_1d_ago, c.channel_id),
            subscriber_7d_ago: find_map!(subscriber_stats_7d_ago, c.channel_id),
            subscriber_30d_ago: find_map!(subscriber_stats_30d_ago, c.channel_id),
            revenue: find_map!(revenue_stats, c.channel_id),
            revenue_1d_ago: find_map!(revenue_stats_1d_ago, c.channel_id),
            revenue_7d_ago: find_map!(revenue_stats_7d_ago, c.channel_id),
            revenue_30d_ago: find_map!(revenue_stats_30d_ago, c.channel_id),
        });

    update_channel_stats(rows, pool).await?;

    Ok(JobResult::Next {
        run: now + Duration::hours(1),
        continuation: None,
    })
}

async fn channel_revenue_stats(
    channels: &[Channel],
    start_at: DateTime<Utc>,
    pool: &PgPool,
) -> anyhow::Result<Vec<ChannelRevenueStatsRow>> {
    let mut _1h_ago = channel_revenue_stats_before(start_at, pool).await?;

    let mut new = list_revenue_by_channel_start_at(start_at, pool).await?;

    let mut results = Vec::with_capacity(channels.len());

    for channel in channels {
        let mut result = _1h_ago
            .iter()
            .position(|r| r.channel_id == channel.channel_id)
            .map(|i| _1h_ago.swap_remove(i).value)
            .unwrap_or_default();

        let mut i = 0;
        while i < new.len() {
            if new[i].channel_id != channel.channel_id {
                i += 1;
                continue;
            }

            let val = new.swap_remove(i);
            let amount = val.amount.and_then(|s| s.parse::<f32>().ok());
            let code = val
                .symbol
                .and_then(|s| currency_symbol_to_code(&s).map(|s| s.to_string()));

            if let (Some(amount), Some(code)) = (amount, code) {
                *result.entry(code).or_default() += amount;
            }
        }

        if !result.is_empty() {
            results.push(ChannelRevenueStatsRow {
                channel_id: channel.channel_id,
                value: result,
            })
        }
    }

    Ok(results)
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

        let response =
            integration_youtube::data_api::channels::list_channels(&channel_ids, client).await?;

        for item in response.items {
            let channel = chunk.iter().find(|ch| ch.platform_id == item.id);

            let Some(channel) = channel else {
                tracing::warn!("Unknown youtube channel id {}", item.id);
                continue;
            };

            view_stats.push(AddChannelViewStatsRow {
                channel_id: channel.channel_id,
                value: item.statistics.view_count.parse().unwrap_or_default(),
            });

            subscribe_stats.push(AddChannelSubscriberStatsRow {
                channel_id: channel.channel_id,
                value: item.statistics.subscriber_count.parse().unwrap_or_default(),
            });
        }
    }

    Ok((view_stats, subscribe_stats))
}

async fn bilibili_channels_stats(
    channels: &[Channel],
    client: &Client,
) -> anyhow::Result<(
    Vec<AddChannelViewStatsRow>,
    Vec<AddChannelSubscriberStatsRow>,
)> {
    let mut view_stats = Vec::with_capacity(channels.len());
    let mut subscribe_stats = Vec::with_capacity(channels.len());

    for channel in channels {
        match try_join!(
            integration_bilibili::channels::channel_subscribers(&channel.platform_id, client),
            integration_bilibili::channels::channel_views(&channel.platform_id, client),
        ) {
            Ok((subscribers, views)) => {
                view_stats.push(AddChannelViewStatsRow {
                    channel_id: channel.channel_id,
                    value: views,
                });

                subscribe_stats.push(AddChannelSubscriberStatsRow {
                    channel_id: channel.channel_id,
                    value: subscribers,
                });
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to get channel stats vtuber_id={} platform=bilibili platform_id={}: {err}",
                    channel.vtuber_id,
                    channel.platform_id
                );
            }
        }
    }

    Ok((view_stats, subscribe_stats))
}
