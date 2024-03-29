use std::env;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use vtstats_utils::send_request;

#[derive(Serialize)]
pub struct SendMessageRequestBody {
    pub chat_id: i64,
    pub parse_mode: MessageParserMode,
    pub text: String,
}

#[derive(Serialize)]
pub struct EditMessageRequestBody {
    pub chat_id: i64,
    pub message_id: i64,
    pub parse_mode: MessageParserMode,
    pub text: String,
}

#[derive(Serialize)]
pub enum MessageParserMode {
    HTML,
    Markdown,
    MarkdownV2,
}

#[derive(Deserialize)]
pub struct MessageResponse {
    pub ok: bool,
    pub result: Message,
}

#[derive(Deserialize)]
pub struct Message {
    pub message_id: i64,
    // "sender_chat": {
    //   "id": i64,
    //   "title": String,
    //   "username": String,
    //   "type":  “private”, “group”, “supergroup” or “channel”
    // },
    // "chat": {
    //   "id": i64,
    //   "title": String,
    //   "username": String,
    //   "type":  “private”, “group”, “supergroup” or “channel”
    // },
    // "date": i64,
    // "text": String
}

pub async fn send_message(
    message: SendMessageRequestBody,
    client: &Client,
) -> anyhow::Result<Message> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        &env::var("TELEGRAM_BOT_TOKEN")?
    );

    let req = client.post(url).form(&message);

    let res = send_request!(req, "/bot:token/sendMessage")?;

    let json: MessageResponse = res.json().await?;

    Ok(json.result)
}

pub async fn edit_message(
    message: EditMessageRequestBody,
    client: &Client,
) -> anyhow::Result<Message> {
    let url = format!(
        "https://api.telegram.org/bot{}/editMessageText",
        &env::var("TELEGRAM_BOT_TOKEN")?
    );

    let req = client.post(url).form(&message);

    let res = send_request!(req, "/bot:token/editMessageText")?;

    let json: MessageResponse = res.json().await?;

    Ok(json.result)
}
