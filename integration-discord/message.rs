use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use vtstats_utils::send_request;

/// https://discord.com/developers/docs/resources/channel#message-object
#[derive(Deserialize)]
pub struct Message {
    pub id: String,
}

/// https://discord.com/developers/docs/resources/channel#embed-object
#[derive(Serialize, Default, Clone)]
#[skip_serializing_none]
pub struct Embed {
    pub timestamp: Option<String>,
    pub title: Option<String>,
    pub color: Option<usize>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub author: Option<EmbedAuthor>,
    pub footer: Option<EmbedFooter>,
    pub image: Option<EmbedImage>,
    pub thumbnail: Option<EmbedThumbnail>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<EmbedField>,
}

#[derive(Serialize, Clone)]
pub struct EmbedThumbnail {
    pub url: String,
}

/// https://discord.com/developers/docs/resources/channel#embed-object-embed-author-structure
#[derive(Serialize, Clone)]
pub struct EmbedAuthor {
    pub name: String,
    pub url: String,
}

/// https://discord.com/developers/docs/resources/channel#embed-object-embed-footer-structure
#[derive(Serialize, Clone)]
pub struct EmbedFooter {
    pub text: String,
}

/// https://discord.com/developers/docs/resources/channel#embed-object-embed-image-structure
#[derive(Serialize, Clone)]
pub struct EmbedImage {
    pub url: String,
    pub height: Option<usize>,
    pub width: Option<usize>,
}

/// https://discord.com/developers/docs/resources/channel#embed-object-embed-field-structure
#[derive(Serialize, Clone)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

/// https://discord.com/developers/docs/resources/channel#create-message
#[derive(Serialize, Clone)]
pub struct CreateMessageRequest {
    #[serde(skip)]
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<Embed>,
}

#[derive(Serialize, Clone)]
pub struct MessageReference {
    pub message_id: String,
    pub fail_if_not_exists: bool,
}

impl CreateMessageRequest {
    pub async fn send(&self, client: &Client) -> anyhow::Result<String> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            self.channel_id
        );

        let req = client.post(url).json(&self).header(
            reqwest::header::AUTHORIZATION,
            format!("Bot {}", std::env::var("DISCORD_BOT_TOKEN").unwrap()),
        );

        let res = send_request!(req, "/api/v10/channels/:channel_id/messages")?;

        let json: Message = res.json().await?;

        Ok(json.id)
    }
}

// https://discord.com/developers/docs/resources/channel#edit-message
#[derive(Serialize)]
pub struct EditMessageRequest {
    #[serde(skip)]
    pub channel_id: String,
    #[serde(skip)]
    pub message_id: String,

    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<Embed>,
}

impl EditMessageRequest {
    pub async fn send(&self, client: &Client) -> anyhow::Result<String> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages/{}",
            self.channel_id, self.message_id
        );

        let req = client.patch(url).json(&self).header(
            reqwest::header::AUTHORIZATION,
            format!("Bot {}", std::env::var("DISCORD_BOT_TOKEN").unwrap()),
        );

        let res = send_request!(req, "/api/v10/channels/:channel_id/messages/:message_id")?;

        let json: Message = res.json().await?;

        Ok(json.id)
    }
}
