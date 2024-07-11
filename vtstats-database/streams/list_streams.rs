use chrono::serde::{ts_milliseconds, ts_milliseconds_option};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::{PgPool, Result};

use crate::channels::Platform;

type UtcTime = DateTime<Utc>;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub platform: Platform,
    pub platform_id: String,
    pub stream_id: i32,
    pub channel_id: i32,
    pub title: String,
    pub vtuber_id: String,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub schedule_time: Option<UtcTime>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub start_time: Option<UtcTime>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub end_time: Option<UtcTime>,
    #[serde(default)]
    pub viewer_avg: Option<i32>,
    #[serde(default)]
    pub viewer_max: Option<i32>,
    #[serde(default)]
    pub like_max: Option<i32>,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: UtcTime,
    pub status: StreamStatus,
}

#[derive(Debug, sqlx::Type, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "stream_status", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum StreamStatus {
    #[default]
    #[serde(alias = "scheduled")]
    Scheduled,
    #[serde(alias = "live")]
    Live,
    #[serde(alias = "ended")]
    Ended,
}

impl StreamStatus {
    /// used in meilisearch
    pub fn lowercase(&self) -> &'static str {
        match self {
            StreamStatus::Scheduled => "scheduled",
            StreamStatus::Live => "live",
            StreamStatus::Ended => "ended",
        }
    }
}

#[derive(Debug, Default)]
pub enum Column {
    #[default]
    StartTime,
    EndTime,
    ScheduleTime,
    UpdatedAt,
}

impl Column {
    /// used in meilisearch
    pub fn camelcase(&self) -> &'static str {
        match self {
            Column::ScheduleTime => "scheduleTime",
            Column::EndTime => "endTime",
            Column::StartTime => "startTime",
            Column::UpdatedAt => "updatedAt",
        }
    }
}

#[derive(Debug, Default)]
pub enum Ordering {
    Asc,
    #[default]
    Desc,
}

impl Ordering {
    /// used in meilisearch
    pub fn lowercase(&self) -> &'static str {
        match self {
            Ordering::Asc => "asc",
            Ordering::Desc => "desc",
        }
    }
}

pub async fn find_missing_stream_id(
    platform_ids: Vec<String>,
    pool: &PgPool,
) -> Result<Vec<String>> {
    let query = sqlx::query!(
        "SELECT platform_id FROM streams WHERE platform_id = ANY($1)",
        &platform_ids
    )
    .fetch_all(pool);

    let ids = crate::otel::execute_query!("SELECT", "streams", query)?;

    Ok(platform_ids
        .into_iter()
        .filter(|id| ids.iter().all(|record| &record.platform_id != id))
        .collect())
}
