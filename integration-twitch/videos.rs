use std::env;

use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{header::AUTHORIZATION, Client, Url};
use serde::Deserialize;
use vtstats_utils::send_request;

#[derive(Deserialize)]
pub struct ListVideoResponse {
    pub data: Vec<Video>,
    #[serde(default)]
    pub pagination: Pagination,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    #[serde(rename = "stream_id")]
    pub stream_id: String,
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_login")]
    pub user_login: String,
    #[serde(rename = "user_name")]
    pub user_name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "published_at")]
    pub published_at: DateTime<Utc>,
    pub url: String,
    #[serde(rename = "thumbnail_url")]
    pub thumbnail_url: String,
    pub viewable: String,
    #[serde(rename = "view_count")]
    pub view_count: i64,
    pub language: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub duration: String,
}

#[derive(Deserialize, Default)]
pub struct Pagination {
    pub cursor: Option<String>,
}

pub async fn list_videos(
    user_id: String,
    after: Option<String>,
    token: &str,
    client: &Client,
) -> Result<ListVideoResponse> {
    let url = if let Some(after) = after {
        Url::parse_with_params(
            "https://api.twitch.tv/helix/videos",
            &[
                ("type", "archive"),
                ("user_id", &user_id),
                ("after", &after),
                ("first", "100"),
            ],
        )
    } else {
        Url::parse_with_params(
            "https://api.twitch.tv/helix/videos",
            &[("type", "archive"), ("user_id", &user_id), ("first", "100")],
        )
    }?;

    let req = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header("Client-Id", env::var("TWITCH_CLIENT_ID")?);

    let res = send_request!(req)?;

    let list: ListVideoResponse = res.json().await?;

    Ok(list)
}
