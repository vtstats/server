use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use vtstats_database::channels::{CreateChannel, Platform};
use vtstats_utils::context::AppContext;

use crate::error::ApiResult;

use super::ActionResponse;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub vtuber_id: String,
    pub kind: Option<String>,
    pub platform: Platform,
    pub platform_id: String,
}

pub async fn create_channel(
    State(state): State<AppContext>,
    Json(payload): Json<Payload>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = CreateChannel {
        platform: payload.platform,
        platform_id: payload.platform_id,
        vtuber_id: payload.vtuber_id.clone(),
        kind: payload.kind,
    }
    .execute(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ActionResponse {
            msg: format!("Channel {} was created.", channel_id),
        }),
    ))
}
