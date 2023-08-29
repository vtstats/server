use vtstats_database::{groups::Group, PgPool};
use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{Reply, Response},
};

use crate::reject::WarpError;

use super::ActionResponse;

pub async fn update_groups(pool: PgPool, groups: Vec<Group>) -> Result<Response, Rejection> {
    vtstats_database::groups::update_groups(groups, pool)
        .await
        .map_err(WarpError::from)?;

    Ok(warp::reply::with_status(
        warp::reply::json(&ActionResponse {
            msg: format!("Groups was updated."),
        }),
        StatusCode::OK,
    )
    .into_response())
}
