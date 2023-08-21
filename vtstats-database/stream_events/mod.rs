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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeSuperChatDonationValue {
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
pub struct YoutubeSuperStickerDonationValue {
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
pub struct YoutubeNewMemberDonationValue {
    pub message: String,
    pub author_name: String,
    #[serde(default)]
    pub author_badges: Option<String>,
    pub author_channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeMemberMilestoneDonationValue {
    pub author_name: String,
    #[serde(default)]
    pub author_badges: Option<String>,
    pub author_channel_id: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StreamEventValue {
    YoutubeSuperChat(YoutubeSuperChatDonationValue),
    YoutubeSuperSticker(YoutubeSuperStickerDonationValue),
    YoutubeNewMember(YoutubeNewMemberDonationValue),
    YoutubeMemberMilestone(YoutubeMemberMilestoneDonationValue),
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
        };

        Ok(StreamEvent { time, kind, value })
    }
}
