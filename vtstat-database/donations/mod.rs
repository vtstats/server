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
    #[serde(rename = "author.name")]
    pub author_name: String,
    #[serde(rename = "author.badges")]
    pub author_badges: String,
    #[serde(rename = "author.channel_id")]
    pub author_channel_id: String,
    #[serde(rename = "paid.amount")]
    pub paid_amount: String,
    #[serde(rename = "paid.currency_code")]
    pub paid_currency_code: String,
    #[serde(rename = "paid.color")]
    pub paid_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeSuperStickerDonationValue {
    pub message: String,
    #[serde(rename = "author.name")]
    pub author_name: String,
    #[serde(rename = "author.badges")]
    pub author_badges: String,
    #[serde(rename = "author.channel_id")]
    pub author_channel_id: String,
    #[serde(rename = "paid.amount")]
    pub paid_amount: String,
    #[serde(rename = "paid.currency_code")]
    pub paid_currency_code: String,
    #[serde(rename = "paid.color")]
    pub paid_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeNewMemberDonationValue {
    pub message: String,
    #[serde(rename = "author.name")]
    pub author_name: String,
    #[serde(rename = "author.badges")]
    pub author_badges: String,
    #[serde(rename = "author.channel_id")]
    pub author_channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeMemberMilestoneDonationValue {
    pub author_name: String,
    #[serde(rename = "author.badges")]
    pub author_badges: String,
    #[serde(rename = "author.channel_id")]
    pub author_channel_id: String,
}

#[derive(Debug, Serialize)]
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
                DonationKind::YoutubeSuperChat => DonationValue::YoutubeSuperChat(
                    row.try_get::<Json<YoutubeSuperChatDonationValue>, _>("value")?
                        .0,
                ),
                DonationKind::YoutubeSuperSticker => DonationValue::YoutubeSuperSticker(
                    row.try_get::<Json<YoutubeSuperStickerDonationValue>, _>("value")?
                        .0,
                ),
                DonationKind::YoutubeNewMember => DonationValue::YoutubeNewMember(
                    row.try_get::<Json<YoutubeNewMemberDonationValue>, _>("value")?
                        .0,
                ),
                DonationKind::YoutubeMemberMilestone => DonationValue::YoutubeMemberMilestone(
                    row.try_get::<Json<YoutubeMemberMilestoneDonationValue>, _>("value")?
                        .0,
                ),
            },
        })
    }
}
