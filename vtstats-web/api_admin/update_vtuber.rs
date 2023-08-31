use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use warp::{reply::Response, Rejection, Reply};

use vtstats_database::{vtubers::UpsertVTuber, PgPool};

use crate::reject::WarpError;

use super::ActionResponse;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVTuberPayload {
    pub vtuber_id: String,
    pub native_name: String,
    #[serde(default)]
    pub english_name: Option<String>,
    #[serde(default)]
    pub japanese_name: Option<String>,
    #[serde(default)]
    pub twitter_username: Option<String>,
    #[serde(default, with = "ts_milliseconds_option")]
    pub retired_at: Option<DateTime<Utc>>,
}

pub async fn update_vtuber(
    pool: PgPool,
    payload: UpdateVTuberPayload,
) -> Result<Response, Rejection> {
    UpsertVTuber {
        vtuber_id: payload.vtuber_id.clone(),
        native_name: payload.native_name,
        english_name: payload.english_name,
        japanese_name: payload.japanese_name,
        twitter_username: payload.twitter_username,
        retired_at: payload.retired_at,
        thumbnail_url: None,
    }
    .execute(&pool)
    .await
    .map_err(WarpError::from)?;

    Ok(warp::reply::with_status(
        warp::reply::json(&ActionResponse {
            msg: format!("VTuber {:?} was updated.", payload.vtuber_id),
        }),
        StatusCode::OK,
    )
    .into_response())
}
