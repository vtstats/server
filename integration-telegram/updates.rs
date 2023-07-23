use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Update {
    pub update_id: i64,
    pub message: Message,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub message_id: i64,
    pub text: String,
    pub chat: Chat,
}

#[derive(Deserialize, Debug)]
pub struct Chat {
    pub id: i64,
}

#[derive(Serialize, Debug)]
pub struct UpdateResponse {
    pub method: &'static str,
    pub parse_mode: &'static str,
    pub chat_id: i64,
    pub text: String,
}
