use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use vtstat_database::{channel_stats as db, PgPool};
use warp::{reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    id: i32,
    #[serde(default, with = "ts_milliseconds_option")]
    start_at: Option<DateTime<Utc>>,
    #[serde(default, with = "ts_milliseconds_option")]
    end_at: Option<DateTime<Utc>>,
}

pub async fn channel_subscriber_stats(
    query: ReqQuery,
    pool: PgPool,
) -> Result<Response, Rejection> {
    let res = db::channel_subscriber_stats(query.id, query.end_at, query.start_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&res).into_response())
}

pub async fn channel_view_stats(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    let res = db::channel_view_stats(query.id, query.end_at, query.start_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&res).into_response())
}
