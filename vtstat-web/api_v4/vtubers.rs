use serde::Serialize;
use vtstat_database::{
    channels::{list_channels, Channel},
    vtubers::{list_vtubers, VTuber},
    PgPool,
};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[derive(Serialize)]
pub struct Res {
    vtubers: Vec<VTuber>,
    channels: Vec<Channel>,
}

pub async fn list_vtubers_and_channels(pool: PgPool) -> Result<Response, Rejection> {
    let vtubers = list_vtubers(&pool).await.map_err(Into::<WarpError>::into)?;

    let channels = list_channels(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&Res { channels, vtubers }).into_response())
}
