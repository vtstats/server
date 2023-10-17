mod proto;
mod request;
mod response;

use chrono::Utc;
use reqwest::{Client, Url};
use std::env;

use self::proto::get_continuation;
use self::request::Request;
use vtstats_utils::send_request;

use super::context::Context;

pub use response::Response;

pub async fn updated_metadata(
    video_id: &str,
    client: &Client,
) -> anyhow::Result<reqwest::Response> {
    let now = Utc::now().timestamp();

    let continuation = get_continuation(video_id, now)?;

    updated_metadata_with_continuation(&continuation, client).await
}

pub async fn updated_metadata_with_continuation(
    continuation: &str,
    client: &Client,
) -> anyhow::Result<reqwest::Response> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/updated_metadata",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        continuation,
    });

    let res = send_request!(req)?;

    Ok(res)
}
