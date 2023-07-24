use vtstat_database::PgPool;
use warp::{reply::Response, Rejection, Reply};

use crate::{api_admin::ActionResponse, reject::WarpError};

pub async fn re_run_job(job_id: i32, pool: PgPool) -> Result<Response, Rejection> {
    vtstat_database::jobs::re_run_job(job_id, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&ActionResponse {
        msg: format!("Job {job_id} was re-run."),
    })
    .into_response())
}
