use axum::{extract::State, http::header::CACHE_CONTROL, response::IntoResponse, Json};

use vtstats_database::exchange_rates::list_exchange_rates;

use crate::{error::ApiResult, AppContext};

pub async fn exchange_rates(State(state): State<AppContext>) -> ApiResult<impl IntoResponse> {
    let res = list_exchange_rates(&state.pool).await?;

    Ok((
        [(CACHE_CONTROL, "public, max-age=864000")], // 10 days
        Json(res),
    ))
}
