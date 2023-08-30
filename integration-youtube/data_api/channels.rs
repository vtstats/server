use std::env;

use chrono::{Timelike, Utc};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

#[derive(Deserialize, Debug)]
pub struct YouTubeChannelsListResponse {
    pub items: Vec<YouTubeChannel>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct YouTubeChannel {
    pub id: String,
    pub statistics: YouTubeChannelStatistics,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeChannelStatistics {
    #[serde(default)]
    pub view_count: String,
    // according to https://github.com/PoiScript/HoloStats/issues/582
    // subscriber_count may be empty in some cases
    #[serde(default)]
    pub subscriber_count: String,
}

pub async fn list_channels(
    id: &str,
    client: &Client,
) -> anyhow::Result<YouTubeChannelsListResponse> {
    let keys = env::var("YOUTUBE_API_KEYS")?;
    let keys: Vec<_> = keys.split(',').collect();
    let key = keys[(Utc::now().hour() as usize) % keys.len()];

    let url = Url::parse_with_params(
        "https://www.googleapis.com/youtube/v3/channels",
        &[
            ("part", "statistics"),
            ("fields", "items(id,statistics(viewCount,subscriberCount))"),
            ("maxResults", "50"),
            ("key", key),
            ("id", id),
        ],
    )?;

    let req = client.get(url);

    let res = send_request!(req)?;

    let json: YouTubeChannelsListResponse = res.json().await?;

    Ok(json)
}
