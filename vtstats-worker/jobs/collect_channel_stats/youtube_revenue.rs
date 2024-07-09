use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

use vtstats_database::{
    channel_stats::{channel_revenue_stats_insert, channel_stats_summary, ChannelStatsKind},
    channels::Channel,
    stream_events::list_youtube_channel_revenue_events,
};
use vtstats_utils::{context::AppContext, currency::currency_symbol_to_code};

pub async fn run(
    channels: &[Channel],
    time: DateTime<Utc>,
    ctx: &AppContext,
) -> anyhow::Result<()> {
    let channel_ids: Vec<_> = channels.iter().map(|c| c.channel_id).collect();

    let revenue_stats =
        channel_stats_summary(&channel_ids, ChannelStatsKind::Revenue, &ctx.search).await?;

    let mut revenue_stats = revenue_stats
        .into_iter()
        .map(|s| {
            if let Some(value) = s.value {
                serde_json::from_value(value).map(|v| (s.channel_id, v))
            } else {
                Ok((s.channel_id, HashMap::new()))
            }
        })
        .collect::<Result<Vec<(i32, HashMap<String, Decimal>)>, _>>()?;

    let revenue_events =
        list_youtube_channel_revenue_events(time - Duration::hours(1), &ctx.pool).await?;

    for event in revenue_events {
        let Some(amount) = event.amount.and_then(|s| s.parse::<Decimal>().ok()) else {
            continue;
        };

        let Some(code) = event
            .symbol
            .and_then(|s| currency_symbol_to_code(&s).map(|s| s.to_string()))
        else {
            continue;
        };

        if let Some((_, map)) = revenue_stats.iter_mut().find(|s| s.0 == event.channel_id) {
            map.entry(code)
                .and_modify(|e| *e += amount)
                .or_insert(amount);
        }
    }

    for (channel_id, value) in revenue_stats {
        channel_revenue_stats_insert(time, channel_id, value, &ctx.pool, &ctx.search).await?;
    }

    Ok(())
}
