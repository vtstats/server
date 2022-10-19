mod list_job;
mod pull_job;
mod push_job;
mod update_job;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Json, FromRow, Row};

pub use self::list_job::ListJobsQuery;
pub use self::pull_job::PullJobQuery;
pub use self::push_job::PushJobQuery;
pub use self::update_job::UpdateJobQuery;

#[derive(sqlx::Type, Debug, PartialEq, Eq)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Success,
    Failed,
}

#[derive(sqlx::Type)]
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
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct UpsertYoutubeStreamJobPayload {
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
}

pub struct Job {
    pub job_id: i32,
    pub continuation: Option<String>,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub payload: JobPayload,
}

impl FromRow<'_, PgRow> for Job {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Job {
            job_id: row.try_get("job_id")?,
            continuation: row.try_get("continuation")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            last_run: row.try_get("last_run")?,
            next_run: row.try_get("next_run")?,
            payload: match row.try_get::<JobKind, _>("kind")? {
                JobKind::HealthCheck => JobPayload::HealthCheck,
                JobKind::RefreshYoutubeRss => JobPayload::RefreshYoutubeRss,
                JobKind::SubscribeYoutubePubsub => JobPayload::SubscribeYoutubePubsub,
                JobKind::UpdateYoutubeChannelViewAndSubscriber => {
                    JobPayload::UpdateYoutubeChannelViewAndSubscriber
                }
                JobKind::UpdateBilibiliChannelViewAndSubscriber => {
                    JobPayload::UpdateBilibiliChannelViewAndSubscriber
                }
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
            },
        })
    }
}
