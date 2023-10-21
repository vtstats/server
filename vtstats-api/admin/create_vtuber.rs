use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use integration_s3::upload_file;
use integration_youtube::youtubei;
use vtstats_database::{
    channels::{CreateChannel, Platform},
    vtubers::UpsertVTuber,
    PgPool,
};
use vtstats_utils::send_request;

use crate::error::ApiResult;

use super::ActionResponse;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVTuberPayload {
    pub vtuber_id: String,
    pub native_name: String,
    #[serde(default)]
    pub english_name: Option<String>,
    #[serde(default)]
    pub japanese_name: Option<String>,
    #[serde(default)]
    pub twitter_username: Option<String>,
    pub youtube_channel_id: String,
    #[serde(default, with = "ts_milliseconds_option")]
    pub retired_at: Option<DateTime<Utc>>,
}

pub async fn create_vtuber(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateVTuberPayload>,
) -> ApiResult<impl IntoResponse> {
    let client = vtstats_utils::reqwest::new()?;

    let mut channel = youtubei::browse_channel(&payload.youtube_channel_id, &client).await?;

    let mut thumbnail_url = channel
        .metadata
        .channel_metadata_renderer
        .avatar
        .thumbnails
        .pop()
        .map(|t| t.url);

    if let Some(url) = thumbnail_url.take() {
        if let Ok(url) = upload_thumbnail(&url, &payload.vtuber_id, &client).await {
            thumbnail_url = Some(url);
        }
    }

    let mut tx = pool.begin().await?;

    UpsertVTuber {
        vtuber_id: payload.vtuber_id.clone(),
        native_name: payload.native_name,
        english_name: payload.english_name,
        japanese_name: payload.japanese_name,
        twitter_username: payload.twitter_username,
        thumbnail_url,
        retired_at: payload.retired_at,
    }
    .execute(&mut *tx)
    .await?;

    CreateChannel {
        platform: Platform::Youtube,
        platform_id: payload.youtube_channel_id,
        vtuber_id: payload.vtuber_id.clone(),
    }
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(ActionResponse {
            msg: format!("VTuber {:?} was created.", payload.vtuber_id),
        }),
    ))
}

async fn upload_thumbnail(url: &str, id: &str, client: &Client) -> anyhow::Result<String> {
    let req = client.get(url);

    let res = send_request!(req)?;

    let file = res.bytes().await?;

    upload_file(&format!("thumbnail/{}.jpg", id), file, "image/jpg", client).await
}
