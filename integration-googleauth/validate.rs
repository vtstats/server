use axum::{
    body::HttpBody,
    http::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;
use std::{env, sync::Arc};
use tokio::sync::Mutex;

use vtstats_utils::send_request;

#[derive(Deserialize)]
struct GoogleCertsKey {
    keys: Vec<Cert>,
}

#[derive(Deserialize)]
struct Cert {
    n: String,
    e: String,
    kid: String,
}

pub struct GoogleCerts {
    keys: Vec<Cert>,
    expire: DateTime<Utc>,
    client: Client,
}

impl GoogleCerts {
    fn new() -> Self {
        GoogleCerts {
            keys: vec![],
            expire: Utc::now(),
            client: vtstats_utils::reqwest::new().expect("create http client"),
        }
    }

    fn is_expired(&self) -> bool {
        self.keys.is_empty() || self.expire < Utc::now()
    }

    async fn refresh(&mut self) -> anyhow::Result<()> {
        let req = self
            .client
            .get("https://www.googleapis.com/oauth2/v3/certs");

        let res = send_request!(req)?;

        let max_age = res
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(get_max_age)
            .unwrap_or(60 * 60); // 1 hour by default

        self.keys = res.json::<GoogleCertsKey>().await?.keys;
        self.expire = Utc::now() + Duration::seconds(max_age);

        Ok(())
    }

    fn validate(&mut self, token: &str) -> anyhow::Result<Claims> {
        let header = decode_header(token)?;

        let Some(kid) = header.kid else {
            anyhow::bail!("`kid` not found in token header")
        };

        let Some(jwk) = &self.keys.iter().find(|k| k.kid == kid) else {
            anyhow::bail!("Cannot find cert with `kid`: {kid}")
        };

        let mut validation = Validation::new(Algorithm::RS256);

        validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
        validation.set_audience(&[&std::env::var("GOOGLE_CLIENT_ID").unwrap()]);

        let token = decode::<Claims>(
            token,
            &DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?,
            &validation,
        )?;

        Ok(token.claims)
    }
}

#[derive(Deserialize, Debug)]
struct Claims {
    pub email: String,
    pub email_verified: bool,
}

static GOOGLE_CERTS: Lazy<Arc<Mutex<GoogleCerts>>> =
    Lazy::new(|| Arc::new(Mutex::new(GoogleCerts::new())));

pub async fn verify<B: HttpBody>(request: Request<B>, next: Next<B>) -> Response {
    let Some(auth) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        tracing::warn!("Header authorization is not present");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let mut certs = GOOGLE_CERTS.lock().await;

    if certs.is_expired() && certs.refresh().await.is_err() {
        tracing::warn!("Failed to fetch google certs keys");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let Ok(claims) = certs.validate(auth) else {
        tracing::warn!("Failed to validate claims");
        return StatusCode::FORBIDDEN.into_response();
    };

    let Ok(emails) = env::var("ADMIN_USER_EMAIL") else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if !claims.email_verified || emails.split(',').all(|email| email != claims.email) {
        tracing::warn!("Not an admin user");
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await
}

fn get_max_age(value: &HeaderValue) -> Option<i64> {
    let needle = b"max-age=";
    let value = value.as_bytes();

    let index = value
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|i| i + needle.len())?;

    let end = value[index..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map(|i| i + index)
        .unwrap_or(value.len());

    String::from_utf8_lossy(&value[index..end])
        .parse::<i64>()
        .ok()
}

#[test]
fn test_get_max_age() {
    assert_eq!(
        get_max_age(&HeaderValue::from_static(
            "public, max-age=22517, must-revalidate, no-transform"
        )),
        Some(22517)
    );

    assert_eq!(
        get_max_age(&HeaderValue::from_static("max-age=604800")),
        Some(604800)
    );

    assert_eq!(
        get_max_age(&HeaderValue::from_static("public, max-age=604800")),
        Some(604800)
    );

    assert_eq!(get_max_age(&HeaderValue::from_static("public")), None);
    assert_eq!(get_max_age(&HeaderValue::from_static("max-age=")), None);
    assert_eq!(get_max_age(&HeaderValue::from_static("max-age=foo")), None);
}
