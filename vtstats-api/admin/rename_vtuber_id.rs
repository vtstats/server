use axum::{extract::State, response::IntoResponse, Json};
use vtstats_database::PgPool;

use crate::{admin::ActionResponse, error::ApiResult};

#[derive(serde::Deserialize)]
pub struct RenameBody {
    before: String,
    after: String,
}

pub async fn rename_vtuber_id(
    State(pool): State<PgPool>,
    Json(body): Json<RenameBody>,
) -> ApiResult<impl IntoResponse> {
    vtstats_database::vtubers::alert_vtuber_id(&body.before, &body.after, pool).await?;

    Ok(Json(ActionResponse {
        msg: format!("VTuber {:?} was renamed to {:?}.", body.before, body.after),
    }))
}
