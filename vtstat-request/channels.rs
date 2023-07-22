use anyhow::Result;
use chrono::{Timelike, Utc};
use reqwest::{header::COOKIE, Url};
use serde::{Deserialize, Serialize};
use std::env;

use super::RequestHub;
use vtstat_utils::instrument_send;

#[derive(Debug)]
pub struct Channel {
    pub id: String,
    pub view_count: i32,
    pub subscriber_count: i32,
}

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
    pub view_count: String,
    // according to https://github.com/PoiScript/HoloStats/issues/582
    // subscriber_count may be empty in some cases
    #[serde(default)]
    pub subscriber_count: String,
}

#[derive(Deserialize)]
pub struct BilibiliUpstatResponse {
    pub data: BilibiliUpstatData,
}

#[derive(Deserialize, Debug)]
pub struct BilibiliUpstatData {
    pub archive: BilibiliUpstatDataArchive,
}

#[derive(Deserialize, Debug)]
pub struct BilibiliUpstatDataArchive {
    pub view: i32,
}

#[derive(Deserialize)]
pub struct BilibiliStatResponse {
    pub data: BilibiliStatData,
}

#[derive(Deserialize, Debug)]
pub struct BilibiliStatData {
    pub follower: i32,
}

impl RequestHub {
    pub async fn youtube_channels(&self, id: &str) -> Result<YouTubeChannelsListResponse> {
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

        let req = self.client.get(url);

        let res = instrument_send(&self.client, req).await?;

        let json: YouTubeChannelsListResponse = res.json().await?;

        Ok(json)
    }

    pub async fn bilibili_stat(&self, id: &str) -> Result<BilibiliStatData> {
        let url =
            Url::parse_with_params("http://api.bilibili.com/x/relation/stat", &[("vmid", id)])?;

        let req = self.client.get(url);

        let res = instrument_send(&self.client, req).await?;

        let json: BilibiliStatResponse = res.json().await?;

        Ok(json.data)
    }

    pub async fn bilibili_upstat(&self, id: &str) -> Result<BilibiliUpstatData> {
        let url = Url::parse_with_params("http://api.bilibili.com/x/space/upstat", &[("mid", id)])?;

        let req = self
            .client
            .get(url)
            .header(COOKIE, env::var("BILIBILI_COOKIE")?);

        let res = instrument_send(&self.client, req).await?;

        let json: BilibiliUpstatResponse = res.json().await?;

        Ok(json.data)
    }
}
