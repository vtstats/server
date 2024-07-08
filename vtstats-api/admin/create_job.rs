use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use vtstats_database::jobs::{JobPayload, PushJobQuery};

use crate::{admin::ActionResponse, error::ApiResult, AppContext};

#[derive(Deserialize)]
#[serde(rename = "UPPER_CASE")]
#[serde(tag = "kind")]
pub enum Payload {
    HealthCheck,
    RefreshYoutubeRss,
    SubscribeYoutubePubsub,
    UpdateChannelStats,
}

pub async fn create_job(
    State(state): State<AppContext>,
    Json(payload): Json<Payload>,
) -> ApiResult<impl IntoResponse> {
    let job_id = PushJobQuery {
        next_run: Some(Utc::now()),
        payload: match payload {
            Payload::HealthCheck => JobPayload::HealthCheck,
            Payload::RefreshYoutubeRss => JobPayload::RefreshYoutubeRss,
            Payload::SubscribeYoutubePubsub => JobPayload::SubscribeYoutubePubsub,
            Payload::UpdateChannelStats => JobPayload::UpdateChannelStats,
        },
    }
    .execute(&state.pool)
    .await?;

    Ok(Json(ActionResponse {
        msg: format!("Job#{job_id} created."),
    }))
}
