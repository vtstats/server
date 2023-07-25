use chrono::{Duration, DurationRound, Utc};
use integration_youtube::data_api::channels::list_channels;
use vtstat_database::{
    channel_stats::{
        AddChannelSubscriberStatsQuery, AddChannelSubscriberStatsRow, AddChannelViewStatsQuery,
        AddChannelViewStatsRow,
    },
    channels::list_youtube_channels,
    PgPool,
};
use vtstat_request::RequestHub;

use super::JobResult;

pub async fn execute(pool: &PgPool, hub: RequestHub) -> anyhow::Result<JobResult> {
    let now = Utc::now().duration_trunc(Duration::hours(1)).unwrap();

    let channels = list_youtube_channels(pool).await?;

    let mut channel_view_stats = Vec::with_capacity(channels.len());
    let mut channel_subscribe_stats = Vec::with_capacity(channels.len());

    // youtube limits 50 channels per request
    for chunk in channels.chunks(50) {
        let channel_ids = chunk.iter().fold(String::new(), |mut acc, channel| {
            if !acc.is_empty() {
                acc.push(',')
            }
            acc.push_str(&channel.platform_id);
            acc
        });

        let response = list_channels(&channel_ids, &hub.client).await?;

        for item in response.items {
            let channel = chunk.iter().find(|ch| ch.platform_id == item.id);

            let Some(channel) = channel else {
                tracing::warn!("Unknown youtube channel id {}", item.id);
                continue;
            };

            channel_view_stats.push(AddChannelViewStatsRow {
                channel_id: channel.channel_id,
                count: item.statistics.view_count.parse().unwrap_or_default(),
            });

            channel_subscribe_stats.push(AddChannelSubscriberStatsRow {
                channel_id: channel.channel_id,
                count: item.statistics.subscriber_count.parse().unwrap_or_default(),
            });
        }
    }

    if !channel_view_stats.is_empty() {
        AddChannelViewStatsQuery {
            time: now,
            rows: channel_view_stats,
        }
        .execute(pool)
        .await?;
    }

    if !channel_subscribe_stats.is_empty() {
        AddChannelSubscriberStatsQuery {
            time: now,
            rows: channel_subscribe_stats,
        }
        .execute(pool)
        .await?;
    }

    Ok(JobResult::Next {
        run: now + Duration::hours(1),
        continuation: None,
    })
}
