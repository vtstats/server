mod request;
mod response;

use std::env;

use reqwest::{Client, Url};

use self::request::Request;
use self::response::Response;
use super::context::Context;

use vtstats_utils::send_request;

pub async fn browse_channel(channel_id: &str, client: &Client) -> anyhow::Result<Response> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/browse",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        browse_id: channel_id,
    });

    let res = send_request!(req)?;

    let json = res.json::<Response>().await?;

    Ok(json)
}
