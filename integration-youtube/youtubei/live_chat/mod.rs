mod proto;
mod request;
pub mod response;

use anyhow::Result;
use reqwest::{Client, Url};
use vtstats_utils::instrument_send;

use self::proto::get_continuation;
use self::request::Request;
pub use self::response::*;
use std::env;

use super::context::Context;

pub async fn youtube_live_chat(
    channel_id: &str,
    stream_id: &str,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    let continuation = get_continuation(channel_id, stream_id)?;

    youtube_live_chat_with_continuation(continuation, client).await
}

pub async fn youtube_live_chat_with_continuation(
    continuation: String,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    let url = Url::parse_with_params(
        "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat",
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        continuation: &continuation,
    });

    let res = instrument_send(client, req).await?;

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
