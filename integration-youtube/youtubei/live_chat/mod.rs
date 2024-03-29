mod proto;
mod request;
pub mod response;

use anyhow::Result;
use chrono::Duration;
use reqwest::{Client, Url};
use std::env;
use vtstats_utils::send_request;

use proto::{get_continuation, get_replay_continuation};
use request::Request;
use response::*;

pub use response::{LiveChatMessage, MemberMessageType, PaidMessageType};

use self::request::CurrentPlayerState;

use super::context::Context;

pub async fn youtube_live_chat(
    channel_id: &str,
    stream_id: &str,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    send_live_chat_request(
        "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat",
        get_continuation(channel_id, stream_id)?,
        None,
        client,
    )
    .await
}

pub async fn youtube_live_chat_with_continuation(
    continuation: String,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    send_live_chat_request(
        "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat",
        continuation,
        None,
        client,
    )
    .await
}

pub async fn replay_live_chat(
    channel_id: &str,
    stream_id: &str,
    offset: Option<Duration>,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    send_live_chat_request(
        "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat_replay",
        get_replay_continuation(channel_id, stream_id)?,
        offset,
        client,
    )
    .await
}

pub async fn replay_live_chat_with_continuation(
    continuation: String,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    send_live_chat_request(
        "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat_replay",
        continuation,
        None,
        client,
    )
    .await
}

async fn send_live_chat_request(
    url: &str,
    continuation: String,
    offset: Option<Duration>,
    client: &Client,
) -> Result<(Vec<LiveChatMessage>, Option<Continuation>)> {
    let url = Url::parse_with_params(
        url,
        &[
            ("prettyPrint", "false"),
            ("key", &env::var("INNERTUBE_API_KEY")?),
        ],
    )?;

    let req = client.post(url).json(&Request {
        context: Context::new()?,
        continuation: &continuation,
        current_player_state: offset.map(|s| CurrentPlayerState {
            player_offset_ms: s.num_milliseconds().to_string(),
        }),
    });

    let res = send_request!(req)?;

    let json: Response = res.json().await?;

    if let Some(contents) = json.continuation_contents {
        let mut messages = vec![];
        for action in contents.live_chat_continuation.actions {
            LiveChatMessage::from_action(action, &mut messages);
        }
        let continuation = contents
            .live_chat_continuation
            .continuations
            .into_iter()
            .next();
        Ok((messages, continuation))
    } else {
        Ok((Vec::new(), None))
    }
}
