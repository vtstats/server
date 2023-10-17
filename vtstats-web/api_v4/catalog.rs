use serde::Serialize;
use vtstats_database::{
    channels::{list_channels, Channel},
    groups::{list_groups, Group},
    vtubers::{list_vtubers, VTuber},
    PgPool,
};
use warp::http::header::{CACHE_CONTROL, VARY};
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

    let res = Catalog {
        channels,
        vtubers,
        groups,
    };

    Ok(warp::reply::with_header(
        warp::reply::with_header(warp::reply::json(&res), VARY, "Origin"),
        CACHE_CONTROL,
        "max-age=3600", // 1 hour
    )
    .into_response())
}
