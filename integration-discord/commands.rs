use reqwest::{header::AUTHORIZATION, Client, Result};
use serde::{Deserialize, Serialize};

use vtstats_utils::send_request;

#[derive(Deserialize)]
pub struct Command {
    id: String,
}

#[derive(Serialize)]
pub struct CommandOption {
    name: String,
    description: String,
    #[serde(rename = "type")]
    ty: usize,
}

impl CommandOption {
    pub const fn string(name: String, description: String) -> CommandOption {
        CommandOption {
            name,
            description,
            ty: 3,
        }
    }

    pub const fn integer(name: String, description: String) -> CommandOption {
        CommandOption {
            name,
            description,
            ty: 4,
        }
    }
}

/// https://discord.com/developers/docs/interactions/application-commands#create-global-application-command
#[derive(Serialize)]
pub struct CreateCommand {
    #[serde(skip)]
    pub application_id: String,

    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub ty: usize,
    pub options: Vec<CommandOption>,
}

impl CreateCommand {
    pub async fn execute(&self, client: &Client) -> Result<String> {
        let url = format!(
            "https://discord.com/api/v10/applications/{}/commands",
            self.application_id
        );

        let req = client.post(url).json(&self).header(
            AUTHORIZATION,
            format!("Bot {}", std::env::var("DISCORD_BOT_TOKEN").unwrap()),
        );

        let res = send_request!(req, "/api/v10/applications/:application_id/commands")?;

        let json: Command = res.json().await?;

        Ok(json.id)
    }
}
