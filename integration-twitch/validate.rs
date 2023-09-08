use bytes::Bytes;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use warp::http::header::HeaderValue;
use warp::{reject::Rejection, Filter};

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

pub fn validate() -> impl Filter<Extract = (Notification,), Error = Rejection> + Clone {
    warp::header::value("Twitch-Eventsub-Message-Id")
        .and(warp::header::optional::<String>(
            "Twitch-Eventsub-Message-Timestamp",
        ))
        .and(warp::header::header("Twitch-Eventsub-Message-Signature"))
        .and(warp::header::header("Twitch-Eventsub-Message-Type"))
        .and(warp::body::bytes())
        .and_then(inner)
}

async fn inner(
    msg_id: HeaderValue,
    msg_timestamp: Option<String>,
    msg_signature: HeaderValue,
    msg_type: HeaderValue,
    body: Bytes,
) -> Result<Notification, Rejection> {
    let Ok(key) = env::var("TWITCH_WEBHOOK_SECRET") else {
        return Err(warp::reject());
    };

    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(msg_id.as_bytes());
    mac.update(msg_timestamp.unwrap_or_default().as_bytes());
    mac.update(&body);

    let expected = mac.finalize().into_bytes();
    let expected = hex::encode(expected);

    if expected.as_bytes() != &msg_signature.as_bytes()["sha256=".len()..] {
        return Err(warp::reject());
    }

    match msg_type.as_bytes() {
        b"notification" => Ok(Notification::Event(serde_json::from_slice(&body).unwrap())),
        b"webhook_callback_verification" => Ok(Notification::Verification(
            serde_json::from_slice(&body).unwrap(),
        )),
        b"revocation" => Ok(Notification::Revocation(
            serde_json::from_slice(&body).unwrap(),
        )),
        _ => Err(warp::reject()),
    }
}
