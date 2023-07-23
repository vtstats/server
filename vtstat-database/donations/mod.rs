pub mod add_donation;
pub mod list_donations;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{postgres::PgRow, types::Json, FromRow, Row};

pub use self::add_donation::*;
pub use self::list_donations::*;

#[derive(sqlx::Type)]
#[sqlx(type_name = "donation_kind", rename_all = "snake_case")]
pub enum DonationKind {
    YoutubeSuperChat,
    YoutubeSuperSticker,
    YoutubeNewMember,
    YoutubeMemberMilestone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeSuperChatDonationValue {
    pub message: String,
    pub author_name: String,
    pub author_badges: String,
    pub author_channel_id: String,
    pub paid_amount: String,
    pub paid_currency_code: String,
    pub paid_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeSuperStickerDonationValue {
    pub message: String,
    pub author_name: String,
    pub author_badges: String,
    pub author_channel_id: String,
    pub paid_amount: String,
    pub paid_currency_code: String,
    pub paid_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeNewMemberDonationValue {
    pub message: String,
    pub author_name: String,
    pub author_badges: String,
    pub author_channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeMemberMilestoneDonationValue {
    pub author_name: String,
    pub author_badges: String,
    pub author_channel_id: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum DonationValue {
    YoutubeSuperChat(YoutubeSuperChatDonationValue),
    YoutubeSuperSticker(YoutubeSuperStickerDonationValue),
    YoutubeNewMember(YoutubeNewMemberDonationValue),
    YoutubeMemberMilestone(YoutubeMemberMilestoneDonationValue),
}

#[derive(Debug)]
pub struct Donation {
    pub time: DateTime<Utc>,
    pub value: DonationValue,
}

impl FromRow<'_, PgRow> for Donation {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Donation {
            time: row.try_get("time")?,
            value: match row.try_get::<DonationKind, _>("kind")? {
                DonationKind::YoutubeSuperChat => {
                    DonationValue::YoutubeSuperChat(row.try_get::<Json<_>, _>("value")?.0)
                }
                DonationKind::YoutubeSuperSticker => {
                    DonationValue::YoutubeSuperSticker(row.try_get::<Json<_>, _>("value")?.0)
                }
                DonationKind::YoutubeNewMember => {
                    DonationValue::YoutubeNewMember(row.try_get::<Json<_>, _>("value")?.0)
                }
                DonationKind::YoutubeMemberMilestone => {
                    DonationValue::YoutubeMemberMilestone(row.try_get::<Json<_>, _>("value")?.0)
                }
            },
        })
    }
}
