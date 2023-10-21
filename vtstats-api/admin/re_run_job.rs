use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use vtstats_database::PgPool;

use crate::{admin::ActionResponse, error::ApiResult};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReRuneJobPayload {
    job_id: i32,
}

pub async fn re_run_job(
    State(pool): State<PgPool>,
    Json(payload): Json<ReRuneJobPayload>,
) -> ApiResult<impl IntoResponse> {
    vtstats_database::jobs::re_run_job(payload.job_id, &pool).await?;

    Ok(Json(ActionResponse {
        msg: format!("Job {} was re-run.", payload.job_id),
    }))
}
