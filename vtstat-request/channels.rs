use anyhow::Result;
use reqwest::{header::COOKIE, Url};
use serde::Deserialize;
use std::env;

use super::RequestHub;
use vtstat_utils::instrument_send;

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
