use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

use vtstats_database::{
    channel_stats_summary::{self, ChannelStatsKind},
    PgPool,
};

use crate::error::ApiResult;

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, i32>")]
    channel_ids: Vec<i32>,
    kind: ChannelStatsKind,
}

pub async fn channel_stats_summary(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<impl IntoResponse> {
    let channels = channel_stats_summary::list(&query.channel_ids, query.kind, &pool).await?;

    Ok(Json(channels))
}
