mod request;
mod response;

use reqwest::{Client, Url};
use std::env;
use vtstats_utils::instrument_send;

use crate::youtubei::context::Context;
use request::Request;
use response::Response;

pub async fn player(video_id: &str, client: &Client) -> anyhow::Result<Response> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/player",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        video_id,
    });

    let res = instrument_send(client, req).await?;

    let json = res.json::<Response>().await?;

    Ok(json)
}
