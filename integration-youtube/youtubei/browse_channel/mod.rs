mod request;
mod response;

use std::env;

use reqwest::{Client, Url};

use self::request::{Client as Client_, Context, Request};
use self::response::Response;

pub async fn browse_channel(channel_id: &str) -> anyhow::Result<Response> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/browse",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let client = Client::new();

    let response = client
        .post(url)
        .json(&Request {
            context: Context {
                client: Client_ {
                    language: "en",
                    client_name: &env::var("INNERTUBE_CLIENT_NAME")?,
                    client_version: &env::var("INNERTUBE_CLIENT_VERSION")?,
                },
            },
            browse_id: channel_id,
        })
        .send()
        .await?;

    let json = response.json::<Response>().await?;

    Ok(json)
}
