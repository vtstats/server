use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use tracing::Span;
use vtstats_database::{stream_stats as db, PgPool};

use crate::error::ApiResult;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    stream_id: i32,
}

pub async fn stream_viewer_stats(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<impl IntoResponse> {
    let stats = db::stream_viewer_stats(query.stream_id, &pool).await?;

    Span::current().record("stream_id", query.stream_id);

    Ok(Json(stats))
}

pub async fn stream_chat_stats(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<impl IntoResponse> {
    let stats = db::stream_chat_stats(query.stream_id, &pool).await?;

    Span::current().record("stream_id", query.stream_id);

    Ok(Json(stats))
}
