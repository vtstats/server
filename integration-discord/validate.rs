use axum::{
    body::{self, BoxBody, Full},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use once_cell::sync::Lazy;

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
pub async fn verify(req: Request<BoxBody>, next: Next<BoxBody>) -> Response {
    let (parts, body) = req.into_parts();

    let Some(sign) = parts.headers.get("X-Signature-Ed25519") else {
        tracing::warn!("Header X-Signature-Ed25519 is not present");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let Some(timestamp) = parts.headers.get("X-Signature-Timestamp") else {
        tracing::warn!("Header X-Signature-Timestamp is not present");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let bytes = match hyper::body::to_bytes(body).await {
        Ok(x) => x,
        Err(err) => {
            tracing::error!("{}", err.to_string());
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(&bytes);

    let sign = hex::decode(sign).unwrap();

    if Signature::from_slice(sign.as_slice())
        .and_then(|sign| VERIFYING_KEY.verify(&message, &sign))
        .is_err()
    {
        tracing::warn!("Invalid request signature");
        return StatusCode::FORBIDDEN.into_response();
    }

    let req = Request::from_parts(parts, body::boxed(Full::from(bytes)));

    next.run(req).await
}

#[tokio::test]
async fn test_verify() {
    use axum::{body::Body, routing::post, Router};
    use tower::{ServiceBuilder, ServiceExt};
    use tower_http::ServiceBuilderExt;

    let layer = ServiceBuilder::new()
        .map_request_body(axum::body::boxed)
        .layer(axum::middleware::from_fn(verify));

    let app = Router::new()
        .route("/", post(|| async move { "hello" }))
        .layer(layer);

    let req = |sign: Option<&str>, timestamp: Option<&str>, body: Option<&str>| -> Request<Body> {
        let mut req = Request::builder().uri("/");

        if let Some(sign) = sign {
            req = req.header("X-Signature-Ed25519", sign.to_string())
        }

        if let Some(timestamp) = timestamp {
            req = req.header("X-Signature-Timestamp", timestamp.to_string())
        }

        req.body(Body::from(body.unwrap_or_default().to_string()))
            .unwrap()
    };

    let invalid_request = [
        (req(None, None, None), StatusCode::BAD_REQUEST),
        (req(Some(""), None, None), StatusCode::BAD_REQUEST),
        (req(Some(""), Some(""), None), StatusCode::FORBIDDEN),
    ];

    for (req, status) in invalid_request {
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), status);
    }
}
