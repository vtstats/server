mod create_channel;
mod list_channels;

pub use create_channel::*;
pub use list_channels::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub channel_id: i32,
    pub platform_id: String,
    pub vtuber_id: String,
    pub platform: Platform,
}

#[derive(sqlx::Type, Serialize, Debug, Deserialize, Clone, Copy, PartialEq, Default)]
#[sqlx(type_name = "platform", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    #[default]
    Youtube,
    Bilibili,
    Twitch,
}
