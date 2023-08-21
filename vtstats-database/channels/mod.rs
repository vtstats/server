mod create_channel;
mod list_channels;
mod update_channel_stats;

pub use create_channel::*;
pub use list_channels::*;
pub use update_channel_stats::*;

use serde::{Deserialize, Serialize};
use sqlx::{types::Json, types::JsonValue};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub channel_id: i32,
    pub platform_id: String,
    pub vtuber_id: String,
    pub platform: Platform,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelWithStats {
    pub vtuber_id: String,
    pub platform: Platform,
    pub view: i32,
    pub view_1d_ago: i32,
    pub view_7d_ago: i32,
    pub view_30d_ago: i32,
    pub subscriber: i32,
    pub subscriber_1d_ago: i32,
    pub subscriber_7d_ago: i32,
    pub subscriber_30d_ago: i32,
    pub revenue: Json<JsonValue>,
    pub revenue_1d_ago: Json<JsonValue>,
    pub revenue_7d_ago: Json<JsonValue>,
    pub revenue_30d_ago: Json<JsonValue>,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "platform", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    Youtube,
    Bilibili,
    Twitch,
}
