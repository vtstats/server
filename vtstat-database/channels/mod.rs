mod list_channels;

pub use self::list_channels::*;

pub struct Channel {
    pub channel_id: i32,
    pub platform_id: String,
    pub vtuber_id: String,
}
