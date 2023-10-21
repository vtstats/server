use axum::{extract::State, http::header::CACHE_CONTROL, response::IntoResponse, Json};
use serde::Serialize;
use vtstats_database::{
    channels::{list_channels, Channel},
    groups::{list_groups, Group},
    vtubers::{list_vtubers, VTuber},
    PgPool,
};

use crate::error::ApiResult;

#[derive(Serialize)]
pub struct Catalog {
    vtubers: Vec<VTuber>,
    channels: Vec<Channel>,
    groups: Vec<Group>,
}

pub async fn catalog(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let vtubers = list_vtubers(&pool).await?;

    let channels = list_channels(&pool).await?;

    let groups = list_groups(&pool).await?;

    let res = Catalog {
        channels,
        vtubers,
        groups,
    };

    Ok((
        [(CACHE_CONTROL, "max-age=3600")], // 1 hour
        Json(res),
    ))
}
