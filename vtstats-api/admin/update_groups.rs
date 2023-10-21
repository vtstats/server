use axum::{extract::State, response::IntoResponse, Json};
use vtstats_database::{groups::Group, PgPool};

use crate::error::ApiResult;

use super::ActionResponse;

pub async fn update_groups(
    State(pool): State<PgPool>,
    Json(groups): Json<Vec<Group>>,
) -> ApiResult<impl IntoResponse> {
    vtstats_database::groups::update_groups(groups, pool).await?;

    Ok(Json(ActionResponse {
        msg: "Groups was updated.".to_string(),
    }))
}
