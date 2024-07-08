use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

use vtstats_database::streams as db;

use crate::{error::ApiResult, AppContext};

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    channel_ids: Vec<i32>,
}

pub async fn stream_times(
    Query(query): Query<ReqQuery>,
    State(state): State<AppContext>,
) -> ApiResult<impl IntoResponse> {
    let times = db::stream_times(&query.channel_ids, &state.pool).await?;

    Ok(Json(times))
}
