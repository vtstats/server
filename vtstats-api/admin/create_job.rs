use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use vtstats_database::{
    jobs::{JobPayload, PushJobQuery},
    PgPool,
};

use crate::{admin::ActionResponse, error::ApiResult};

#[derive(Deserialize)]
#[serde(rename = "UPPER_CASE")]
#[serde(tag = "kind")]
pub enum CreateJobPayload {
    HealthCheck,
    RefreshYoutubeRss,
    SubscribeYoutubePubsub,
    UpdateChannelStats,
}

pub async fn create_job(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateJobPayload>,
) -> ApiResult<impl IntoResponse> {
    let job_id = PushJobQuery {
        next_run: Some(Utc::now()),
        payload: match payload {
            CreateJobPayload::HealthCheck => JobPayload::HealthCheck,
            CreateJobPayload::RefreshYoutubeRss => JobPayload::RefreshYoutubeRss,
            CreateJobPayload::SubscribeYoutubePubsub => JobPayload::SubscribeYoutubePubsub,
            CreateJobPayload::UpdateChannelStats => JobPayload::UpdateChannelStats,
        },
    }
    .execute(&pool)
    .await?;

    Ok(Json(ActionResponse {
        msg: format!("Job#{job_id} created."),
    }))
}
