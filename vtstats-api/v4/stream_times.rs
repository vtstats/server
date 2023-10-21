use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

use vtstats_database::{streams as db, PgPool};

use crate::error::ApiResult;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    channel_ids: Vec<i32>,
}

pub async fn stream_times(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<impl IntoResponse> {
    let times = db::stream_times(&query.channel_ids, &pool).await?;

    Ok(Json(times))
}
