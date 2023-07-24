use std::env;
use std::str::FromStr;

use chrono::{DateTime, Timelike, Utc};
use reqwest::{Client, Url};
use serde::Deserialize;
use vtstat_database::streams::StreamStatus;
use vtstat_utils::instrument_send;

#[derive(Deserialize, Debug)]
pub struct VideosListResponse {
    pub items: Vec<Video>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub snippet: Option<Snippet>,
    pub live_streaming_details: Option<LiveStreamingDetails>,
    pub statistics: Option<Statistics>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub channel_id: String,
    pub title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamingDetails {
    pub actual_start_time: Option<DateTime<Utc>>,
    pub actual_end_time: Option<DateTime<Utc>>,
    pub scheduled_start_time: Option<DateTime<Utc>>,
    pub concurrent_viewers: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub like_count: Option<String>,
}

pub async fn list_videos(id: &str, client: &Client) -> anyhow::Result<Vec<Video>> {
    let keys = env::var("YOUTUBE_API_KEYS")?;
    let keys: Vec<_> = keys.split(',').collect();
    let key = keys[(Utc::now().hour() as usize) % keys.len()];

    let url = Url::parse_with_params(
        "https://www.googleapis.com/youtube/v3/videos",
        &[
            ("part", "id,statistics,liveStreamingDetails,snippet"),
            ("fields", "items(id,statistics(likeCount),liveStreamingDetails(actualStartTime,actualEndTime,scheduledStartTime,concurrentViewers),snippet(title,channelId))"),
            ("maxResults", "50"),
            ("key", key),
            ("id", id),
        ],
    )?;

    let req = client.get(url);

    let res = instrument_send(client, req).await?;

    let json: VideosListResponse = res.json().await?;

    Ok(json.items)
}

#[derive(Debug)]
pub struct Stream {
    pub id: String,
    pub title: String,
    pub channel_id: String,
    pub status: StreamStatus,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub schedule_time: Option<DateTime<Utc>>,
    pub viewers: Option<i32>,
    pub likes: Option<i32>,
}

impl From<Video> for Option<Stream> {
    fn from(video: Video) -> Option<Stream> {
        let snippet = video.snippet?;
        let detail = video.live_streaming_details?;

        Some(Stream {
            id: video.id,
            title: snippet.title,
            channel_id: snippet.channel_id,
            status: if detail.actual_end_time.is_some() {
                StreamStatus::Ended
            } else if detail.actual_start_time.is_some() {
                StreamStatus::Live
            } else {
                StreamStatus::Scheduled
            },
            start_time: detail.actual_start_time,
            end_time: detail.actual_end_time,
            schedule_time: detail.scheduled_start_time,
            viewers: detail
                .concurrent_viewers
                .and_then(|v| i32::from_str(&v).ok()),
            likes: video
                .statistics
                .and_then(|s| s.like_count)
                .and_then(|v| i32::from_str(&v).ok()),
        })
    }
}
