mod proto;
mod request;
pub mod response;

use anyhow::Result;
use reqwest::Url;
use vtstat_utils::instrument_send;

use self::proto::get_continuation;
use self::request::{Client, Context, Request};
pub use self::response::{Continuation, LiveChatMessage, Response};
use std::env;

use super::RequestHub;

impl RequestHub {
    pub async fn youtube_live_chat(
        &self,
        channel_id: &str,
        stream_id: &str,
    ) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
        let continuation = get_continuation(channel_id, stream_id)?;

        self.youtube_live_chat_with_continuation(continuation).await
    }

    pub async fn youtube_live_chat_with_continuation(
        &self,
        continuation: String,
    ) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
        let url = Url::parse_with_params(
            "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat",
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
            continuation: &continuation,
        });

        let res = instrument_send(&self.client, req).await?;

        let json: Response = res.json().await?;

        if let Some(contents) = json.continuation_contents {
            Ok((
                contents
                    .live_chat_continuation
                    .actions
                    .into_iter()
                    .filter_map(LiveChatMessage::from_action)
                    .collect(),
                contents
                    .live_chat_continuation
                    .continuations
                    .into_iter()
                    .next(),
            ))
        } else {
            Ok((Vec::new(), None))
        }
    }
}
