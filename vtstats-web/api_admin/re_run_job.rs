use serde::{Deserialize, Serialize};
use vtstats_database::PgPool;
use warp::{reply::Response, Rejection, Reply};

use crate::{api_admin::ActionResponse, reject::WarpError};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReRuneJobPayload {
    job_id: i32,
}

pub async fn re_run_job(pool: PgPool, payload: ReRuneJobPayload) -> Result<Response, Rejection> {
    vtstats_database::jobs::re_run_job(payload.job_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&ActionResponse {
        msg: format!("Job {} was re-run.", payload.job_id),
    })
    .into_response())
}
