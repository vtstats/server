mod request;
mod response;

use anyhow::Result;
use reqwest::{Client, Url};
use std::env;
use vtstats_utils::send_request;

use request::Request;
use response::Response;

use super::context::Context;

pub async fn browse_channel_live(channel_id: &str, client: &Client) -> Result<Response> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/browse",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        browse_id: Some(channel_id),
        params: Some("EgdzdHJlYW1z8gYECgJ6AA%3D%3D"),
        continuation: None,
    });

    let res = send_request!(req)?;

    let text = res.text().await?;

    Ok(serde_json::from_str(&text)?)
}

pub async fn browse_channel_live_with_continuation(
    continuation: &str,
    client: &Client,
) -> Result<Response> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/browse",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        browse_id: None,
        params: None,
        continuation: Some(continuation),
    });

    let res = send_request!(req)?;

    Ok(res.json().await?)
}
