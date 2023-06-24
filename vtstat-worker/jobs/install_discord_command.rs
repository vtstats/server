use super::JobResult;

use integration_discord::commands::{CommandOption, CreateCommand};
use reqwest::Client;

pub async fn execute(client: &Client) -> anyhow::Result<JobResult> {
    let _ = CreateCommand {
        application_id: std::env::var("DISCOARD_APPLICATION_ID")?,
        description: "".into(),
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
        application_id: std::env::var("DISCOARD_APPLICATION_ID")?,
        description: "".into(),
        name: "list".into(),
        ty: 1,
        options: vec![],
    }
    .execute(client)
    .await?;

    let _ = CreateCommand {
        application_id: std::env::var("DISCOARD_APPLICATION_ID")?,
        description: "".into(),
        name: "remove".into(),
        ty: 1,
        options: vec![CommandOption::integer(
            "subscription_id".into(),
            "ID of subscription".into(),
        )],
    }
    .execute(client)
    .await?;

    Ok(JobResult::Completed)
}
