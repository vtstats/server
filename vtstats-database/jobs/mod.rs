mod list_job;
mod next_queued;
mod pull_job;
mod push_job;
mod re_run;
mod update_job;

use chrono::serde::{ts_milliseconds, ts_milliseconds_option};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, Row};

pub use self::list_job::*;
pub use self::next_queued::*;
pub use self::pull_job::*;
pub use self::push_job::*;
pub use self::re_run::*;
pub use self::update_job::*;

#[derive(sqlx::Type, Debug, PartialEq, Eq, Serialize, Clone, Copy)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Queued,
    Running,
    Success,
    Failed,
}

#[derive(sqlx::Type, Serialize, Clone, Copy)]
#[sqlx(type_name = "job_kind", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobKind {
    HealthCheck,
    RefreshYoutubeRss,
    SubscribeYoutubePubsub,
    #[sqlx(rename = "update_youtube_channel_view_and_subscriber")]
    UpdateChannelStats,
    UpdateCurrencyExchangeRate,
    UpsertYoutubeStream,
    CollectYoutubeStreamMetadata,
    CollectTwitchStreamMetadata,
    SendNotification,
    InstallDiscordCommands,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct UpsertYoutubeStreamJobPayload {
    pub vtuber_id: String,
    pub channel_id: i32,
    pub platform_stream_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct CollectYoutubeStreamMetadataJobPayload {
    pub stream_id: i32,
    pub platform_stream_id: String,
    pub platform_channel_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct CollectTwitchStreamMetadataJobPayload {
    pub stream_id: i32,
    pub platform_stream_id: String,
    pub platform_channel_id: String,
    pub platform_channel_login: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SendNotificationJobPayload {
    /// Unique identifier for vtuber, e.g. 'shirakami-fubuki'
    pub vtuber_id: String,
    /// Always be 'youtube'
    pub stream_platform: String,
    pub stream_platform_id: String,
}

#[derive(Serialize, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum JobPayload {
    HealthCheck,
    RefreshYoutubeRss,
    SubscribeYoutubePubsub,
    UpdateChannelStats,
    UpdateCurrencyExchangeRate,
    UpsertYoutubeStream(UpsertYoutubeStreamJobPayload),
    CollectYoutubeStreamMetadata(CollectYoutubeStreamMetadataJobPayload),
    CollectTwitchStreamMetadata(CollectTwitchStreamMetadataJobPayload),
    SendNotification(SendNotificationJobPayload),
    InstallDiscordCommands,
}

#[derive(Serialize)]
pub struct Job {
    pub job_id: i32,
    pub continuation: Option<String>,
    pub status: JobStatus,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_milliseconds_option")]
    pub last_run: Option<DateTime<Utc>>,
    #[serde(with = "ts_milliseconds_option")]
    pub next_run: Option<DateTime<Utc>>,
    pub payload: JobPayload,
    pub kind: JobKind,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
}

impl JobPayload {
    pub fn kind(&self) -> JobKind {
        match self {
            JobPayload::HealthCheck => JobKind::HealthCheck,
            JobPayload::RefreshYoutubeRss => JobKind::RefreshYoutubeRss,
            JobPayload::SubscribeYoutubePubsub => JobKind::SubscribeYoutubePubsub,
            JobPayload::UpdateChannelStats => JobKind::UpdateChannelStats,
            JobPayload::UpdateCurrencyExchangeRate => JobKind::UpdateCurrencyExchangeRate,
            JobPayload::UpsertYoutubeStream(_) => JobKind::UpsertYoutubeStream,
            JobPayload::CollectYoutubeStreamMetadata(_) => JobKind::CollectYoutubeStreamMetadata,
            JobPayload::CollectTwitchStreamMetadata(_) => JobKind::CollectTwitchStreamMetadata,
            JobPayload::SendNotification(_) => JobKind::SendNotification,
            JobPayload::InstallDiscordCommands => JobKind::InstallDiscordCommands,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            JobPayload::HealthCheck => "health_check",
            JobPayload::RefreshYoutubeRss => "refresh_youtube_rss",
            JobPayload::SubscribeYoutubePubsub => "subscribe_youtube_pubsub",
            JobPayload::UpdateChannelStats => "update_channel_stats",
            JobPayload::UpdateCurrencyExchangeRate => "update_currency_exchange_rate",
            JobPayload::UpsertYoutubeStream(_) => "upsert_youtube_stream",
            JobPayload::CollectYoutubeStreamMetadata(_) => "collect_youtube_stream_metadata",
            JobPayload::CollectTwitchStreamMetadata(_) => "collect_twitch_stream_metadata",
            JobPayload::SendNotification(_) => "send_notification",
            JobPayload::InstallDiscordCommands => "install_discord_commands",
        }
    }
}

impl FromRow<'_, PgRow> for Job {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let kind = row.try_get::<JobKind, _>("kind")?;
        Ok(Job {
            job_id: row.try_get("job_id")?,
            continuation: row.try_get("continuation")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            last_run: row.try_get("last_run")?,
            next_run: row.try_get("next_run")?,
            kind,
            payload: match kind {
                JobKind::HealthCheck => JobPayload::HealthCheck,
                JobKind::RefreshYoutubeRss => JobPayload::RefreshYoutubeRss,
                JobKind::SubscribeYoutubePubsub => JobPayload::SubscribeYoutubePubsub,
                JobKind::UpdateChannelStats => JobPayload::UpdateChannelStats,
                JobKind::UpdateCurrencyExchangeRate => JobPayload::UpdateCurrencyExchangeRate,
                JobKind::UpsertYoutubeStream => {
                    JobPayload::UpsertYoutubeStream(row.try_get::<Json<_>, _>("payload")?.0)
                }
                JobKind::CollectYoutubeStreamMetadata => JobPayload::CollectYoutubeStreamMetadata(
                    row.try_get::<Json<_>, _>("payload")?.0,
                ),
                JobKind::CollectTwitchStreamMetadata => {
                    JobPayload::CollectTwitchStreamMetadata(row.try_get::<Json<_>, _>("payload")?.0)
                }
                JobKind::SendNotification => {
                    JobPayload::SendNotification(row.try_get::<Json<_>, _>("payload")?.0)
                }
                JobKind::InstallDiscordCommands => JobPayload::InstallDiscordCommands,
            },
        })
    }
}
