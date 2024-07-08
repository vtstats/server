use axum::{extract::State, response::IntoResponse, Json};
use vtstats_database::groups::Group;

use crate::{error::ApiResult, AppContext};

use super::ActionResponse;

pub async fn update_groups(
    State(state): State<AppContext>,
    Json(groups): Json<Vec<Group>>,
) -> ApiResult<impl IntoResponse> {
    vtstats_database::groups::update_groups(groups, state.pool).await?;

    Ok(Json(ActionResponse {
        msg: "Groups was updated.".to_string(),
    }))
}
