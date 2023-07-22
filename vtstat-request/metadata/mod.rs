mod proto;
mod request;
mod response;

use chrono::Utc;
use reqwest::Url;
use std::env;

use self::proto::get_continuation;
use self::request::{Client, Context, Request};
use self::response::Response;
use vtstat_utils::instrument_send;

use super::RequestHub;

impl RequestHub {
    pub async fn updated_metadata(&self, video_id: &str) -> anyhow::Result<Response> {
        let now = Utc::now().timestamp();

        let continuation = get_continuation(video_id, now)?;

        self.updated_metadata_with_continuation(&continuation).await
    }

    pub async fn updated_metadata_with_continuation(
        &self,
        continuation: &str,
    ) -> anyhow::Result<Response> {
        let url = Url::parse_with_params(
            "https://www.youtube.com/youtubei/v1/updated_metadata",
            &[
                ("prettyPrint", "false"),
                ("key", &env::var("INNERTUBE_API_KEY")?),
            ],
        )?;

        let req = self.client.post(url).json(&Request {
            context: Context {
                client: Client {
                    language: "en",
                    client_name: &env::var("INNERTUBE_CLIENT_NAME")?,
                    client_version: &env::var("INNERTUBE_CLIENT_VERSION")?,
                },
            },
            continuation,
        });

        let res = instrument_send(&self.client, req).await?;

        let json: Response = res.json().await?;

        Ok(json)
    }
}
