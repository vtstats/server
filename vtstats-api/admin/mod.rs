mod create_job;
mod create_vtuber;
mod re_run_job;
mod rename_vtuber_id;
mod update_groups;
mod update_vtuber;

use axum::{
    extract::{Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};

use integration_googleauth::verify;
use vtstats_database::{
    streams::{Column, ListYouTubeStreamsQuery, Ordering},
    PgPool,
};

use crate::error::ApiResult;

use self::{
    create_job::create_job, create_vtuber::create_vtuber, re_run_job::re_run_job,
    rename_vtuber_id::rename_vtuber_id, update_groups::update_groups, update_vtuber::update_vtuber,
};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        // jobs
        .route("/jobs", get(list_jobs).put(create_job))
        .route("/jobs/re-run", post(re_run_job))
        // streams
        .route("/streams", get(list_streams))
        // notifications
        .route("/notifications", get(list_notifications))
        .route("/subscriptions", get(list_subscriptions))
        // catalog
        .route(
            "/vtubers",
            get(list_vtubers).put(create_vtuber).post(update_vtuber),
        )
        .route("/vtubers/rename", post(rename_vtuber_id))
        .route("/channels", get(list_channels))
        .route("/groups", get(list_groups).post(update_groups))
        .layer(middleware::from_fn(verify))
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct ListParameter {
    #[serde(default, with = "ts_milliseconds_option")]
    end_at: Option<DateTime<Utc>>,
    status: Option<String>,
}

async fn list_groups(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let groups = vtstats_database::groups::list_groups(&pool).await?;
    Ok(Json(groups))
}

async fn list_jobs(
    State(pool): State<PgPool>,
    Query(parameter): Query<ListParameter>,
) -> ApiResult<impl IntoResponse> {
    let jobs = vtstats_database::jobs::list_jobs_order_by_updated_at(
        parameter.status.unwrap_or_else(|| "queued".into()),
        parameter.end_at,
        &pool,
    )
    .await?;
    Ok(Json(jobs))
}

async fn list_streams(
    State(pool): State<PgPool>,
    Query(parameter): Query<ListParameter>,
) -> ApiResult<impl IntoResponse> {
    let status = match parameter.status.as_deref() {
        Some("scheduled") => "scheduled",
        Some("live") => "live",
        _ => "ended",
    };

    let streams = ListYouTubeStreamsQuery {
        limit: Some(24),
        order_by: Some((Column::UpdatedAt, Ordering::Desc)),
        end_at: parameter.end_at.as_ref().map(|dt| (Column::UpdatedAt, dt)),
        status: &[status.into()],
        ..Default::default()
    }
    .execute(&pool)
    .await?;

    Ok(Json(streams))
}

async fn list_channels(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let channels = vtstats_database::channels::list_channels(&pool).await?;
    Ok(Json(channels))
}

async fn list_notifications(
    State(pool): State<PgPool>,
    Query(parameter): Query<ListParameter>,
) -> ApiResult<impl IntoResponse> {
    let notifications = vtstats_database::subscriptions::list(parameter.end_at, &pool).await?;
    Ok(Json(notifications))
}

async fn list_subscriptions(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let subscriptions =
        vtstats_database::subscriptions::list_subscriptions::list_subscriptions(&pool).await?;

    Ok(Json(subscriptions))
}

async fn list_vtubers(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let vtubers = vtstats_database::vtubers::list_vtubers(&pool).await?;

    Ok(Json(vtubers))
}

#[derive(Serialize)]
pub struct ActionResponse {
    msg: String,
}
