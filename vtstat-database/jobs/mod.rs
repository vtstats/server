mod list_job;
mod pull_job;
mod push_job;
mod re_run;
mod update_job;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, Row};

pub use self::list_job::{list_jobs_order_by_updated_at, ListJobsQuery};
pub use self::pull_job::PullJobQuery;
pub use self::push_job::PushJobQuery;
pub use self::re_run::re_run_job;
pub use self::update_job::UpdateJobQuery;

#[derive(sqlx::Type, Debug, PartialEq, Eq, Serialize)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Success,
    Failed,
}

#[derive(sqlx::Type, Serialize, Clone, Copy)]
#[sqlx(type_name = "job_kind", rename_all = "snake_case")]
pub enum JobKind {
    HealthCheck,
    RefreshYoutubeRss,
    SubscribeYoutubePubsub,
    UpdateYoutubeChannelViewAndSubscriber,
    UpdateBilibiliChannelViewAndSubscriber,
    UpdateYoutubeChannelDonation,
    UpdateCurrencyExchangeRate,
    UpsertYoutubeStream,
    CollectYoutubeStreamMetadata,
    CollectYoutubeStreamLiveChat,
    UpdateUpcomingStream,
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
pub struct CollectYoutubeStreamLiveChatJobPayload {
    pub stream_id: i32,
    pub platform_channel_id: String,
    pub platform_stream_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SendNotificationJobPayload {
    /// Either 'discord' or 'telegram'
    pub platform: String,
    /// Unique identifier for vtuber, e.g. 'shirakamifubuki'
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
    UpdateYoutubeChannelViewAndSubscriber,
    UpdateBilibiliChannelViewAndSubscriber,
    UpdateYoutubeChannelDonation,
    UpdateCurrencyExchangeRate,
    UpsertYoutubeStream(UpsertYoutubeStreamJobPayload),
    CollectYoutubeStreamMetadata(CollectYoutubeStreamMetadataJobPayload),
    CollectYoutubeStreamLiveChat(CollectYoutubeStreamLiveChatJobPayload),
    UpdateUpcomingStream,
    SendNotification(SendNotificationJobPayload),
    InstallDiscordCommands,
}

#[derive(Serialize)]
pub struct Job {
    pub job_id: i32,
    pub continuation: Option<String>,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub payload: JobPayload,
    pub kind: JobKind,
    pub updated_at: DateTime<Utc>,
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
                JobKind::UpdateYoutubeChannelViewAndSubscriber => {
                    JobPayload::UpdateYoutubeChannelViewAndSubscriber
                }
                JobKind::UpdateBilibiliChannelViewAndSubscriber => {
                    JobPayload::UpdateBilibiliChannelViewAndSubscriber
                }
                JobKind::UpdateUpcomingStream => JobPayload::UpdateUpcomingStream,
                JobKind::UpdateYoutubeChannelDonation => JobPayload::UpdateYoutubeChannelDonation,
                JobKind::UpdateCurrencyExchangeRate => JobPayload::UpdateCurrencyExchangeRate,
                JobKind::UpsertYoutubeStream => JobPayload::UpsertYoutubeStream(
                    row.try_get::<Json<UpsertYoutubeStreamJobPayload>, _>("payload")?
                        .0,
                ),
                JobKind::CollectYoutubeStreamMetadata => JobPayload::CollectYoutubeStreamMetadata(
                    row.try_get::<Json<CollectYoutubeStreamMetadataJobPayload>, _>("payload")?
                        .0,
                ),
                JobKind::CollectYoutubeStreamLiveChat => JobPayload::CollectYoutubeStreamLiveChat(
                    row.try_get::<Json<CollectYoutubeStreamLiveChatJobPayload>, _>("payload")?
                        .0,
                ),
                JobKind::SendNotification => JobPayload::SendNotification(
                    row.try_get::<Json<SendNotificationJobPayload>, _>("payload")?
                        .0,
                ),
                JobKind::InstallDiscordCommands => JobPayload::InstallDiscordCommands,
            },
        })
    }
}
