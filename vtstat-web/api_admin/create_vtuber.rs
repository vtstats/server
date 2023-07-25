use reqwest::{Client, StatusCode};
use serde::Deserialize;
use warp::{reply::Response, Rejection, Reply};

use integration_s3::upload_file;
use integration_youtube::youtubei;
use vtstat_database::{
    channels::{CreateChannel, Platform},
    vtubers::CreateVTuber,
    PgPool,
};
use vtstat_utils::instrument_send;

use crate::reject::WarpError;

use super::ActionResponse;

#[derive(Deserialize)]
pub struct CreateVTuberPayload {
    pub vtuber_id: String,
    pub native_name: String,
    pub english_name: Option<String>,
    pub japanese_name: Option<String>,
    pub twitter_username: Option<String>,
    pub youtube_channel_id: String,
}

pub async fn create_vtuber(
    pool: PgPool,
    payload: CreateVTuberPayload,
) -> Result<Response, Rejection> {
    let client = Client::new();

    let mut channel = youtubei::browse_channel(&payload.youtube_channel_id, &client)
        .await
        .map_err(WarpError)?;

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

    CreateVTuber {
        vtuber_id: payload.vtuber_id.clone(),
        native_name: payload.native_name,
        english_name: payload.english_name,
        japanese_name: payload.japanese_name,
        twitter_username: payload.twitter_username,
        thumbnail_url,
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    CreateChannel {
        platform: Platform::Youtube,
        platform_id: payload.youtube_channel_id,
        vtuber_id: payload.vtuber_id.clone(),
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::with_status(
        warp::reply::json(&ActionResponse {
            msg: format!("VTuber {:?} was created.", payload.vtuber_id),
        }),
        StatusCode::CREATED,
    )
    .into_response())
}

async fn upload_thumbnail(url: &str, id: &str, client: &Client) -> anyhow::Result<String> {
    let req = client.get(url);

    let res = instrument_send(client, req).await?.error_for_status()?;

    let file = res.bytes().await?;

    upload_file(&format!("thumbnail/{}.jpg", id), file, "image/jpg", client).await
}
