use vtstat_database::{currencies::list_currencies as list_currencies_, PgPool};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

pub async fn list_currencies(pool: PgPool) -> Result<Response, Rejection> {
    let currencies = list_currencies_(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&currencies).into_response())
}
