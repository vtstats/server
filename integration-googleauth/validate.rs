use std::{env, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::{header::HeaderValue, Client};
use serde::Deserialize;
use tokio::sync::Mutex;
use warp::{Filter, Rejection};

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

#[derive(Deserialize, Debug)]
struct Claims {
    pub email: String,
    pub email_verified: bool,
}

impl GoogleCerts {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(GoogleCerts {
            keys: vec![],
            expire: Utc::now(),
            client: Client::new(),
        }))
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

#[derive(Debug, Clone, Copy)]
pub struct NeedLogin;

impl warp::reject::Reject for NeedLogin {}

impl warp::Reply for NeedLogin {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(warp::reply(), warp::http::StatusCode::UNAUTHORIZED)
            .into_response()
    }
}

pub fn validate(
    certs: Arc<Mutex<GoogleCerts>>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::header::optional::<String>(warp::http::header::AUTHORIZATION.as_str())
        .and_then(move |auth: Option<String>| {
            let certs = certs.clone();
            async move {
                let auth = auth.ok_or_else(|| warp::reject::custom(NeedLogin))?;

                let mut certs = certs.lock().await;

                if certs.is_expired() {
                    certs.refresh().await.map_err(|_| warp::reject())?;
                }

                let claims = certs.validate(&auth).map_err(|_| warp::reject())?;

                let emails =
                    env::var("ADMIN_USER_EMAIL").map_err(|_| warp::reject::custom(NeedLogin))?;

                if claims.email_verified && emails.split(',').any(|email| email == claims.email) {
                    return Ok(());
                }

                Err(warp::reject::custom(NeedLogin))
            }
        })
        .untuple_one()
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
