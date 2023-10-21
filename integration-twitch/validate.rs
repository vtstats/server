use axum::{
    async_trait,
    body::{self, BoxBody, Full},
    extract::FromRequest,
    http::Request,
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    response::Response,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;

use crate::subscription::{Event, Subscription};

#[derive(Deserialize, Debug)]
pub struct VerificationChallenge {
    pub challenge: String,
}

pub enum Notification {
    Event(Event),
    Verification(VerificationChallenge),
    Revocation(Subscription),
}

#[async_trait]
impl<S> FromRequest<S, BoxBody> for Notification
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<BoxBody>, _: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let Some(message_type) = parts.headers.get("Twitch-Eventsub-Message-Type") else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let body = match hyper::body::to_bytes(body).await {
            Ok(b) => b,
            Err(err) => {
                tracing::error!("{}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        match message_type.as_bytes() {
            b"notification" => Ok(Notification::Event(serde_json::from_slice(&body).unwrap())),
            b"webhook_callback_verification" => Ok(Notification::Verification(
                serde_json::from_slice(&body).unwrap(),
            )),
            b"revocation" => Ok(Notification::Revocation(
                serde_json::from_slice(&body).unwrap(),
            )),
            _ => Err(StatusCode::BAD_REQUEST.into_response()),
        }
    }
}

pub async fn verify(req: Request<BoxBody>, next: Next<BoxBody>) -> Response {
    let (parts, body) = req.into_parts();

    let Some(message_id) = parts.headers.get("Twitch-Eventsub-Message-Id") else {
        tracing::warn!("Header Twitch-Eventsub-Message-Id is not present");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let Some(message_signature) = parts.headers.get("Twitch-Eventsub-Message-Signature") else {
        tracing::warn!("Header Twitch-Eventsub-Message-Signature is not present");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let Ok(key) = env::var("TWITCH_WEBHOOK_SECRET") else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let message_timestamp = parts.headers.get("Twitch-Eventsub-Message-Timestamp");

    let bytes = match hyper::body::to_bytes(body).await {
        Ok(x) => x,
        Err(err) => {
            tracing::error!("{}", err.to_string());
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(message_id.as_bytes());
    mac.update(message_timestamp.map(|x| x.as_bytes()).unwrap_or_default());
    mac.update(&bytes);

    let expected = hex::encode(mac.finalize().into_bytes());

    if expected.as_bytes() != &message_signature.as_bytes()["sha256=".len()..] {
        tracing::warn!("Invalid request signature");
        return StatusCode::FORBIDDEN.into_response();
    }

    let req = Request::from_parts(parts, body::boxed(Full::from(bytes)));

    next.run(req).await
}
