mod create_channel;
mod list_channels;

pub use self::create_channel::*;
pub use self::list_channels::*;

use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Channel {
    pub channel_id: i32,
    pub platform_id: String,
    pub vtuber_id: String,
    pub platform: Platform,
}

#[derive(sqlx::Type, Serialize, Clone, Copy)]
#[sqlx(type_name = "platform", rename_all = "snake_case")]
pub enum Platform {
    Youtube,
    Bilibili,
    Twitch,
}
