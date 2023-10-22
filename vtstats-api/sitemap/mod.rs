use axum::{
    extract::State, http::header::CONTENT_TYPE, response::IntoResponse, routing::get, Router,
};
use std::fmt::Write;

use tokio::try_join;
use vtstats_database::{
    channels::Platform, streams::list_stream_ids, vtubers::list_vtuber_ids, PgPool,
};

use crate::error::ApiResult;

pub fn router(pool: PgPool) -> Router {
    Router::new().route("/", get(sitemap)).with_state(pool)
}

// Returns a sitemap for crawler like google search
async fn sitemap(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    const HOSTNAME: &str = "https://vt.poi.cat";

    let mut res = String::new();

    let (vtuber_ids, stream_ids) = try_join!(list_vtuber_ids(&pool), list_stream_ids(&pool))?;

    for id in vtuber_ids {
        let _ = writeln!(res, "{HOSTNAME}/vtuber/{id}");
    }

    for record in stream_ids {
        let platform = match record.platform {
            Platform::Youtube => "youtube",
            Platform::Bilibili => "bilibili",
            Platform::Twitch => "twitch",
        };
        let _ = writeln!(res, "{HOSTNAME}/{platform}-stream/{}", record.platform_id);
    }

    Ok(([(CONTENT_TYPE, "text/plain")], res))
}
