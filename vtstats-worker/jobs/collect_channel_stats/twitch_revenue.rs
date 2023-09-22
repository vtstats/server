use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

use vtstats_database::{
    channel_stats_summary::{insert, list, AddChannelStats, ChannelStatsKind},
    channels::Channel,
    stream_events::list_twitch_channel_revenue_events,
    PgPool,
};

pub async fn run(channels: &[Channel], time: DateTime<Utc>, pool: &PgPool) -> anyhow::Result<()> {
    let channel_ids: Vec<_> = channels.iter().map(|c| c.channel_id).collect();

    let revenue_stats = list(&channel_ids, ChannelStatsKind::Revenue, pool).await?;

    let mut revenue_stats = revenue_stats
        .into_iter()
        .map(|s| {
            if s.value.is_null() {
                Ok((s.channel_id, HashMap::new()))
            } else {
                serde_json::from_value(s.value).map(|v| (s.channel_id, v))
            }
        })
        .collect::<Result<Vec<(i32, HashMap<String, Decimal>)>, _>>()?;

    let revenue_events =
        list_twitch_channel_revenue_events(time - Duration::hours(1), pool).await?;

    for event in revenue_events {
        let Some(mut amount) = event.amount.and_then(|s| s.parse::<Decimal>().ok()) else {
            continue;
        };

        let Some(mut code) = event.symbol.map(|s| s.to_string()) else {
            continue;
        };

        if code == "cheering" {
            code = "USD".into();
            amount /= Decimal::ONE_HUNDRED;
        }

        if let Some((_, map)) = revenue_stats.iter_mut().find(|s| s.0 == event.channel_id) {
            map.entry(code)
                .and_modify(|e| *e += amount)
                .or_insert(amount);
        }
    }

    for (channel_id, value) in revenue_stats {
        insert(time, channel_id, AddChannelStats::Revenue(value), pool).await?;
    }

    Ok(())
}
