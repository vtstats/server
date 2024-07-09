use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

use vtstats_database::channel_stats::ChannelStatsKind;

use crate::{error::ApiResult, AppContext};

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
    State(state): State<AppContext>,
) -> ApiResult<impl IntoResponse> {
    use vtstats_database::channel_stats::channel_stats_summary;

    let channels = channel_stats_summary(&query.channel_ids, query.kind, &state.search).await?;

    Ok(Json(channels))
}
