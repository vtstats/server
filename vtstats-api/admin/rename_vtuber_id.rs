use axum::{extract::State, response::IntoResponse, Json};

use crate::{admin::ActionResponse, error::ApiResult, AppContext};

#[derive(serde::Deserialize)]
pub struct Payload {
    before: String,
    after: String,
}

pub async fn rename_vtuber_id(
    State(state): State<AppContext>,
    Json(body): Json<Payload>,
) -> ApiResult<impl IntoResponse> {
    vtstats_database::vtubers::alert_vtuber_id(&body.before, &body.after, state.pool).await?;

    Ok(Json(ActionResponse {
        msg: format!("VTuber {:?} was renamed to {:?}.", body.before, body.after),
    }))
}
