use axum::{
    body::HttpBody,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn verify<B: HttpBody>(req: Request<B>, next: Next<B>) -> Response {
    let Some(expected) = req.headers().get("x-telegram-bot-api-secret-token") else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let Ok(token) = std::env::var("TELEGRAM_SECRET_TOKEN") else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if expected.as_bytes() != token.as_bytes() {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(req).await
}
