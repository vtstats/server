use anyhow::Result;
use reqwest::{header::COOKIE, Client, Url};
use serde::Deserialize;
use std::env;

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

pub async fn channel_subscribers(id: &str, client: &Client) -> Result<i32> {
    let url = Url::parse_with_params("http://api.bilibili.com/x/relation/stat", &[("vmid", id)])?;

    let req = client.get(url).header(COOKIE, env::var("BILIBILI_COOKIE")?);

    let res = instrument_send(&client, req).await?;

    let json: BilibiliStatResponse = res.json().await?;

    Ok(json.data.follower)
}

pub async fn channel_views(id: &str, client: &Client) -> Result<i32> {
    let url = Url::parse_with_params("https://api.bilibili.com/x/space/upstat", &[("mid", id)])?;

    let req = client.get(url).header(COOKIE, env::var("BILIBILI_COOKIE")?);

    let res = instrument_send(&client, req).await?;

    let json: BilibiliUpstatResponse = res.json().await?;

    Ok(json.data.archive.view)
}
