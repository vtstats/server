use reqwest::Client;
use serde::{Deserialize, Serialize};

/// https://discord.com/developers/docs/resources/channel#message-object
#[derive(Deserialize)]
pub struct Message {
    id: String,
}

/// https://discord.com/developers/docs/resources/channel#create-message
#[derive(Serialize)]
pub struct CreateMessageRequest {
    #[serde(skip)]
    channel_id: String,

    content: String,
}

impl CreateMessageRequest {
    pub async fn send(&self, client: &Client) -> anyhow::Result<String> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            self.channel_id
        );

        let req = client.post(url).form(&self);

        let res = req.send().await?;

        let json: Message = res.json().await?;

        Ok(json.id)
    }
}

// https://discord.com/developers/docs/resources/channel#edit-message
#[derive(Serialize)]
pub struct EditMessageRequest {
    #[serde(skip)]
    channel_id: String,

    #[serde(skip)]
    message_id: String,

    content: String,
}

impl EditMessageRequest {
    pub async fn send(&self, client: &Client) -> anyhow::Result<String> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages/{}",
            self.channel_id, self.message_id
        );

        let req = client.patch(url).json(&self).header(
            "Authorization",
            format!("Bot {}", std::env::var("DISCOARD_BOT_TOKEN").unwrap()),
        );

        let res = req.send().await?;

        let json: Message = res.json().await?;

        Ok(json.id)
    }
}
