use bytes::Bytes;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use once_cell::sync::Lazy;
use reqwest::header::HeaderValue;
use warp::{Filter, Rejection};

use crate::interaction::Interaction;

static VERIFYING_KEY: Lazy<VerifyingKey> = Lazy::new(|| {
    let mut keys = [0u8; 32];

    hex::decode_to_slice(
        &std::env::var("DISCORD_APPLICATION_PUBLIC_KEY").unwrap_or_default(),
        &mut keys,
    )
    .unwrap();

    VerifyingKey::from_bytes(&keys).unwrap()
});

/// verify if this request is actually came from discord
///
/// https://discord.com/developers/docs/interactions/receiving-and-responding#security-and-authorization
pub fn validate() -> impl Filter<Extract = (Interaction,), Error = Rejection> + Clone {
    warp::header::value("X-Signature-Ed25519")
        .and(warp::header::value("X-Signature-Timestamp"))
        .and(warp::body::bytes())
        .and_then(
            |sign: HeaderValue, timestamp: HeaderValue, body: Bytes| async move {
                let mut message = timestamp.as_bytes().to_vec();
                message.extend_from_slice(&body);

                let sign = hex::decode(sign).unwrap();

                if Signature::from_slice(sign.as_slice())
                    .and_then(|sign| VERIFYING_KEY.verify(&message, &sign))
                    .is_err()
                {
                    print!("invalid request signature");
                    return Err(warp::reject());
                }

                let json: Interaction = serde_json::from_slice(&body).unwrap();

                Ok(json)
            },
        )
}
