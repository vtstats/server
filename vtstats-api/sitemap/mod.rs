use axum::{
    extract::State, http::header::CONTENT_TYPE, response::IntoResponse, routing::get, Router,
};
use std::fmt::Write;
use vtstats_database::{vtubers::list_vtubers, PgPool};

use crate::error::ApiResult;

pub fn router(pool: PgPool) -> Router {
    Router::new().route("/", get(sitemap)).with_state(pool)
}

// Returns a sitemap for crawler like google search
async fn sitemap(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    const HOSTNAME: &str = "https://vt.poi.cat";

    let mut res = String::new();

    let vtubers = list_vtubers(&pool).await?;

    for vtuber in vtubers {
        let _ = writeln!(res, "{HOSTNAME}/vtuber/{}", vtuber.vtuber_id);
    }

    // TODO streams

    Ok(([(CONTENT_TYPE, "text/plain")], res))
}
