use chrono::{serde::ts_milliseconds, DateTime, Utc};
use meilisearch_sdk::{
    client::Client,
    documents::{DocumentsQuery, DocumentsResults},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::types::JsonValue;

use crate::streams::meilisearch::ArrayFieldFilter;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelStatsKind {
    Subscriber,
    View,
    Revenue,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub channel_id: i32,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub kind: Option<ChannelStatsKind>,
    #[serde(default)]
    pub value: Option<JsonValue>,
    #[serde(default)]
    pub value_1_day_ago: Option<JsonValue>,
    #[serde(default)]
    pub value_7_days_ago: Option<JsonValue>,
    #[serde(default)]
    pub value_30_days_ago: Option<JsonValue>,
}

pub async fn add_or_update(
    document: Document,
    kind: ChannelStatsKind,
    client: &Client,
) -> anyhow::Result<()> {
    let index = client.index(match kind {
        ChannelStatsKind::Subscriber => "channel_subscriber_stats_summary",
        ChannelStatsKind::View => "channel_view_stats_summary",
        ChannelStatsKind::Revenue => "channel_revenue_stats_summary",
    });

    let _task = index
        .add_or_update(&[&document], Some("channelId".into()))
        .await?;

    Ok(())
}

pub async fn list(
    channel_ids: &[i32],
    kind: ChannelStatsKind,
    client: &Client,
) -> anyhow::Result<Vec<Document>> {
    let index = client.index(match kind {
        ChannelStatsKind::Subscriber => "channel_subscriber_stats_summary",
        ChannelStatsKind::View => "channel_view_stats_summary",
        ChannelStatsKind::Revenue => "channel_revenue_stats_summary",
    });

    let mut result: DocumentsResults<_> = DocumentsQuery::new(&index)
        .with_filter(&ArrayFieldFilter("channelId", channel_ids).to_string())
        .with_limit(channel_ids.len())
        .execute::<Document>()
        .await?;

    // TODO: fix
    for r in result.results.iter_mut() {
        r.kind.replace(kind);
    }

    Ok(result.results)
}
