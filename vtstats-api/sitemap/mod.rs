use axum::{
    extract::State, http::header::CONTENT_TYPE, response::IntoResponse, routing::get, Router,
};
use std::fmt::Write;

use vtstats_database::{channels::Platform, streams::list_stream_ids, vtubers::list_vtuber_ids};

use crate::{error::ApiResult, AppContext};

pub fn router(pool: AppContext) -> Router {
    Router::new().route("/", get(sitemap)).with_state(pool)
}

// Returns a sitemap for crawler like google search
async fn sitemap(State(ctx): State<AppContext>) -> ApiResult<impl IntoResponse> {
    const HOSTNAME: &str = "https://vt.poi.cat";

    let vtuber_ids = list_vtuber_ids(&ctx.pool).await?;

    let stream_ids = list_stream_ids(&ctx.search).await?;

    let mut res = String::new();

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
