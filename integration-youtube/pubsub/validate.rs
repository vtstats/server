use axum::{
    body, body::BoxBody, body::Full, http::Request, http::StatusCode, middleware::Next,
    response::IntoResponse, response::Response,
};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::env;

/// verify if this request is came from pubsubhubbub
///
/// https://pubsubhubbub.github.io/PubSubHubbub/pubsubhubbub-core-0.4.html#authednotify
pub async fn verify(req: Request<BoxBody>, next: Next<BoxBody>) -> Response {
    let (parts, body) = req.into_parts();

    let Some(signature) = parts.headers.get("x-hub-signature") else {
        tracing::warn!("Header x-hub-signature is not present");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let Some(key) = env::var("YOUTUBE_PUBSUB_SECRET").ok() else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let bytes = match hyper::body::to_bytes(body).await {
        Ok(x) => x,
        Err(err) => {
            tracing::error!("{}", err.to_string());
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut mac =
        Hmac::<Sha1>::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(&bytes);

    let expected = hex::encode(mac.finalize().into_bytes());

    if expected.as_bytes() != &signature.as_bytes()["sha1=".len()..] {
        tracing::warn!("Invalid request signature");
        return StatusCode::FORBIDDEN.into_response();
    }

    let req = Request::from_parts(parts, body::boxed(Full::from(bytes)));

    next.run(req).await
}
