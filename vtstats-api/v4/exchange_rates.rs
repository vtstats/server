use axum::{extract::State, http::header::CACHE_CONTROL, response::IntoResponse, Json};

use vtstats_database::{exchange_rates::list_exchange_rates, PgPool};

use crate::error::ApiResult;

pub async fn exchange_rates(State(pool): State<PgPool>) -> ApiResult<impl IntoResponse> {
    let res = list_exchange_rates(&pool).await?;

    Ok((
        [(CACHE_CONTROL, "max-age=864000")], // 10 days
        Json(res),
    ))
}
