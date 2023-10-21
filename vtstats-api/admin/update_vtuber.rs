use axum::{extract::State, response::IntoResponse, Json};
use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde::Deserialize;

use vtstats_database::{vtubers::UpsertVTuber, PgPool};

use crate::error::ApiResult;

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
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateVTuberPayload>,
) -> ApiResult<impl IntoResponse> {
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
    .await?;

    Ok(Json(ActionResponse {
        msg: format!("VTuber {:?} was updated.", payload.vtuber_id),
    }))
}
