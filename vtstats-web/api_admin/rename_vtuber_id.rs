use vtstats_database::PgPool;
use warp::{reply::Response, Rejection, Reply};

use crate::{api_admin::ActionResponse, reject::WarpError};

#[derive(serde::Deserialize)]
pub struct RenameBody {
    before: String,
    after: String,
}

pub async fn rename_vtuber_id(pool: PgPool, body: RenameBody) -> Result<Response, Rejection> {
    vtstats_database::vtubers::alert_vtuber_id(&body.before, &body.after, pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&ActionResponse {
        msg: format!("VTuber {:?} was renamed to {:?}.", body.before, body.after),
    })
    .into_response())
}
