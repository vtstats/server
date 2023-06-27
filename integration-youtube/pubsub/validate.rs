use bytes::Bytes;

use hmac::Hmac;
use hmac::Mac;
use sha1::Sha1;
use std::env;
use warp::{Filter, Rejection};

use super::event::Event;

fn generate_signature(data: &Bytes) -> Option<String> {
    let key = env::var("YOUTUBE_PUBSUB_SECRET").ok()?;

    let mut mac =
        Hmac::<Sha1>::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(data);

    let result = mac.finalize().into_bytes();

    Some(hex::encode(result))
}

/// verify if this request is came from pubsubhubbub
///
/// https://pubsubhubbub.github.io/PubSubHubbub/pubsubhubbub-core-0.4.html#authednotify
pub fn validate() -> impl Filter<Extract = (Event,), Error = Rejection> + Copy {
    warp::header::header("x-hub-signature")
        .and(warp::body::bytes())
        .and_then(|sign: String, body: Bytes| async move {
            let Some(expected) = generate_signature(&body) else {
                return Err(warp::reject());
            };

            let found = sign.trim_start_matches("sha1=");

            if expected != found {
                return Err(warp::reject());
            }

            let body = std::str::from_utf8(&body).map_err(|_| warp::reject())?;

            let event: Event = body.parse().map_err(|_| warp::reject())?;

            Ok(event)
        })
}
