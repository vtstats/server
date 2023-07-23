use warp::{Filter, Rejection};

use crate::updates::Update;

pub fn verify_request() -> impl Filter<Extract = (Update,), Error = Rejection> + Copy {
    warp::header::exact(
        "x-telegram-bot-api-secret-token",
        option_env!("TELEGRAM_SECRET_TOKEN").unwrap_or_default(),
    )
    .and(warp::body::json())
}
