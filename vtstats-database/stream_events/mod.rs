mod add_stream_events;
mod list_stream_events;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{postgres::PgRow, types::Json, FromRow, Row};

pub use self::add_stream_events::*;
pub use self::list_stream_events::*;

#[derive(Debug, sqlx::Type, Clone, Copy, Serialize)]
#[sqlx(type_name = "stream_event_kind", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamEventKind {
    YoutubeSuperChat,
    YoutubeSuperSticker,
    YoutubeNewMember,
    YoutubeMemberMilestone,
    TwitchCheering,
    TwitchHyperChat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeSuperChat {
    #[serde(default)]
    pub message: Option<String>,
    pub author_name: String,
    #[serde(default)]
    pub author_badges: Option<String>,
    pub author_channel_id: String,
    pub paid_amount: String,
    pub paid_currency_symbol: String,
    pub paid_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeSuperSticker {
    #[serde(default)]
    pub message: Option<String>,
    pub author_name: String,
    #[serde(default)]
    pub author_badges: Option<String>,
    pub author_channel_id: String,
    pub paid_amount: String,
    pub paid_currency_symbol: String,
    pub paid_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeNewMember {
    pub message: String,
    pub author_name: String,
    #[serde(default)]
    pub author_badges: Option<String>,
    pub author_channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeMemberMilestone {
    pub author_name: String,
    #[serde(default)]
    pub author_badges: Option<String>,
    pub author_channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchCheering {
    pub author_username: String,
    #[serde(default)]
    pub badges: Option<String>,
    pub bits: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchHyperChat {
    pub author_username: String,
    #[serde(default)]
    pub badges: Option<String>,
    pub message: String,
    pub currency_code: String,
    pub level: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StreamEventValue {
    YoutubeSuperChat(YoutubeSuperChat),
    YoutubeSuperSticker(YoutubeSuperSticker),
    YoutubeNewMember(YoutubeNewMember),
    YoutubeMemberMilestone(YoutubeMemberMilestone),
    TwitchCheering(TwitchCheering),
    TwitchHyperChat(TwitchHyperChat),
}

impl StreamEventValue {
    pub fn kind(&self) -> StreamEventKind {
        match self {
            StreamEventValue::YoutubeSuperChat(_) => StreamEventKind::YoutubeSuperChat,
            StreamEventValue::YoutubeSuperSticker(_) => StreamEventKind::YoutubeSuperSticker,
            StreamEventValue::YoutubeNewMember(_) => StreamEventKind::YoutubeNewMember,
            StreamEventValue::YoutubeMemberMilestone(_) => StreamEventKind::YoutubeMemberMilestone,
            StreamEventValue::TwitchCheering(_) => StreamEventKind::TwitchCheering,
            StreamEventValue::TwitchHyperChat(_) => StreamEventKind::TwitchHyperChat,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StreamEvent {
    pub time: DateTime<Utc>,
    pub kind: StreamEventKind,
    pub value: StreamEventValue,
}

impl FromRow<'_, PgRow> for StreamEvent {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let time = row.try_get("time")?;
        let kind = row.try_get::<StreamEventKind, _>("kind")?;
        let value = match kind {
            StreamEventKind::YoutubeSuperChat => {
                StreamEventValue::YoutubeSuperChat(row.try_get::<Json<_>, _>("value")?.0)
            }
            StreamEventKind::YoutubeSuperSticker => {
                StreamEventValue::YoutubeSuperSticker(row.try_get::<Json<_>, _>("value")?.0)
            }
            StreamEventKind::YoutubeNewMember => {
                StreamEventValue::YoutubeNewMember(row.try_get::<Json<_>, _>("value")?.0)
            }
            StreamEventKind::YoutubeMemberMilestone => {
                StreamEventValue::YoutubeMemberMilestone(row.try_get::<Json<_>, _>("value")?.0)
            }
            StreamEventKind::TwitchCheering => {
                StreamEventValue::TwitchCheering(row.try_get::<Json<_>, _>("value")?.0)
            }
            StreamEventKind::TwitchHyperChat => {
                StreamEventValue::TwitchHyperChat(row.try_get::<Json<_>, _>("value")?.0)
            }
        };

        Ok(StreamEvent { time, kind, value })
    }
}
