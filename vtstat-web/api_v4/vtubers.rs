use vtstat_database::{vtubers as db, PgPool};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

pub async fn list_vtubers(pool: PgPool) -> Result<Response, Rejection> {
    let vtubers = db::list_vtubers(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&vtubers).into_response())
}
