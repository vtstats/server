use super::JobResult;

use integration_discord::commands::{CommandOption, CreateCommand};
use reqwest::Client;

pub async fn execute(client: &Client) -> anyhow::Result<JobResult> {
    let _ = CreateCommand {
        application_id: std::env::var("DISCORD_APPLICATION_ID")?,
        description: "Add subscription".into(),
        name: "add".into(),
        ty: 1,
        options: vec![CommandOption::string(
            "vtuber_id".into(),
            "ID of vtuber".into(),
        )],
    }
    .execute(client)
    .await?;

    let _ = CreateCommand {
        application_id: std::env::var("DISCORD_APPLICATION_ID")?,
        description: "List subscriptions of this channel".into(),
        name: "list".into(),
        ty: 1,
        options: vec![],
    }
    .execute(client)
    .await?;

    let _ = CreateCommand {
        application_id: std::env::var("DISCORD_APPLICATION_ID")?,
        description: "List all subscriptions of this servers".into(),
        name: "list_all".into(),
        ty: 1,
        options: vec![],
    }
    .execute(client)
    .await?;

    let _ = CreateCommand {
        application_id: std::env::var("DISCORD_APPLICATION_ID")?,
        description: "Remove subscription".into(),
        name: "remove".into(),
        ty: 1,
        options: vec![CommandOption::string(
            "vtuber_id".into(),
            "ID of vtuber".into(),
        )],
    }
    .execute(client)
    .await?;

    Ok(JobResult::Completed)
}
