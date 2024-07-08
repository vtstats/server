use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{admin::ActionResponse, error::ApiResult, AppContext};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    job_id: i32,
}

pub async fn re_run_job(
    State(state): State<AppContext>,
    Json(payload): Json<Payload>,
) -> ApiResult<impl IntoResponse> {
    vtstats_database::jobs::re_run_job(payload.job_id, &state.pool).await?;

    Ok(Json(ActionResponse {
        msg: format!("Job {} was re-run.", payload.job_id),
    }))
}
