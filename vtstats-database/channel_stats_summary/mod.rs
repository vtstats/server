mod insert;
mod list;

pub use insert::*;
pub use list::*;

use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

#[derive(sqlx::Type, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "channel_stats_kind", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelStatsKind {
    Subscriber,
    View,
    Revenue,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatsSummary {
    pub channel_id: i32,
    pub kind: ChannelStatsKind,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    pub value: JsonValue,
    pub value_1_day_ago: JsonValue,
    pub value_7_days_ago: JsonValue,
    pub value_30_days_ago: JsonValue,
}
