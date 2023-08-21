use serde::Serialize;
use vtstats_database::{
    channels::{list_channels, Channel},
    groups::{list_groups, Group},
    vtubers::{list_vtubers, VTuber},
    PgPool,
};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[derive(Serialize)]
pub struct Catalog {
    vtubers: Vec<VTuber>,
    channels: Vec<Channel>,
    groups: Vec<Group>,
}

pub async fn catalog(pool: PgPool) -> Result<Response, Rejection> {
    let vtubers = list_vtubers(&pool).await.map_err(Into::<WarpError>::into)?;

    let channels = list_channels(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    let groups = list_groups(&pool).await.map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&Catalog {
        channels,
        vtubers,
        groups,
    })
    .into_response())
}
