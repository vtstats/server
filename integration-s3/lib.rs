use anyhow::Result;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Body, Client,
};
use sha2::{Digest, Sha256};
use std::env;

use vtstats_utils::send_request;

pub async fn upload_file<T>(
    filename: &str,
    data: T,
    content_type: &str,
    client: &Client,
) -> Result<String>
where
    T: Into<Body> + AsRef<[u8]>,
{
    let (s3_host, s3_key_id, s3_access_key, s3_region, s3_public_url, s3_bucket) = (
        env::var("S3_HOST")?,
        env::var("S3_KEY_ID")?,
        env::var("S3_ACCESS_KEY")?,
        env::var("S3_REGION")?,
        env::var("S3_PUBLIC_URL")?,
        env::var("S3_BUCKET")?,
    );

    let now = Utc::now();
    let date = now.format("%Y%m%dT%H%M%SZ");
    let today = now.format("%Y%m%d");

    let content_sha256 = Sha256::digest(data.as_ref());
    let content_sha256 = hex::encode(content_sha256);

    // task1
    let canonical_req = format!(
        r#"PUT
/{s3_bucket}/{filename}

host:{s3_host}
x-amz-content-sha256:{content_sha256}
x-amz-date:{date}

host;x-amz-content-sha256;x-amz-date
{content_sha256}"#
    );

    let hashed_canonical_request = Sha256::digest(canonical_req.as_bytes());
    let hashed_canonical_request = hex::encode(hashed_canonical_request);

    // task2
    let scope = format!("{today}/{s3_region}/s3/aws4_request");

    // StringToSign = Algorithm + \n + RequestDateTime + \n + CredentialScope + \n + HashedCanonicalRequest
    let string_to_sign = format!("AWS4-HMAC-SHA256\n{date}\n{scope}\n{hashed_canonical_request}");

    // task3
    macro_rules! hmac_sha256 {
        ($key:expr, $data:expr) => {{
            let mut mac =
                Hmac::<Sha256>::new_from_slice($key).expect("HMAC can take key of any size");
            mac.update($data);
            mac.finalize().into_bytes()
        }};
    }

    // kSecret = your secret access key
    let k_secret = format!("AWS4{s3_access_key}");
    // kDate = HMAC("AWS4" + kSecret, Date)
    let k_date = hmac_sha256!(k_secret.as_bytes(), today.to_string().as_bytes());
    // kRegion = HMAC(kDate, Region)
    let k_region = hmac_sha256!(k_date.as_slice(), s3_region.as_bytes());
    // kService = HMAC(kRegion, Service)
    let k_service = hmac_sha256!(k_region.as_slice(), b"s3");
    // kSigning = HMAC(kService, "aws4_request")
    let k_signing = hmac_sha256!(k_service.as_slice(), b"aws4_request");
    // signature = HexEncode(HMAC(derived signing key, string to sign))
    let signature = hmac_sha256!(k_signing.as_slice(), string_to_sign.as_bytes());
    let signature = hex::encode(signature);

    // task4
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={s3_key_id}/{scope}, \
        SignedHeaders=host;x-amz-content-sha256;x-amz-date, \
        Signature={signature}"
    );

    let s3_url = format!("https://{s3_host}/{s3_bucket}/{filename}");

    let req = client
        .put(s3_url)
        .header(CONTENT_TYPE, content_type)
        .header("x-amz-date", date.to_string())
        .header("x-amz-content-sha256", content_sha256)
        .header(AUTHORIZATION, authorization)
        .body(data);

    send_request!(req, "/:bucket/:filename")?;

    Ok(format!("{s3_public_url}/{filename}"))
}
