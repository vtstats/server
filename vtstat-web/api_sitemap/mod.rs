use std::convert::Into;
use std::env;
use std::fmt::Write;
use vtstat_database::{vtubers::ListVtubersQuery, PgPool};
use warp::{Filter, Rejection};

use crate::filters::with_pool;
use crate::reject::WarpError;

const PAGES: &[&str] = &[
    "youtube-channel",
    "bilibili-channel",
    "youtube-stream",
    "youtube-schedule-stream",
    "settings",
];

// Returns a sitemap for crawler like google search
async fn sitemap_get(pool: PgPool) -> Result<impl warp::Reply, Rejection> {
    let mut res = String::new();

    let hostname = env::var("SERVER_HOSTNAME").map_err(Into::<WarpError>::into)?;

    for page in PAGES {
        let _ = writeln!(res, "https://{hostname}/{page}");
    }

    let vtubers = ListVtubersQuery
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    for vtuber in vtubers {
        let _ = writeln!(res, "https://{hostname}/vtuber/{}", vtuber.vtuber_id);
    }

    // TODO streams

    Ok(res)
}

pub fn sitemap(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("sitemap")
        .and(warp::get())
        .and(with_pool(pool))
        .and_then(sitemap_get)
}
