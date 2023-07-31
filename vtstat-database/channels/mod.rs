mod create_channel;
mod list_channels;
mod update_channel_stats;

pub use create_channel::*;
pub use list_channels::*;
pub use update_channel_stats::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Channel {
    pub channel_id: i32,
    pub platform_id: String,
    pub vtuber_id: String,
    pub platform: Platform,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "platform", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    Youtube,
    Bilibili,
    Twitch,
}
