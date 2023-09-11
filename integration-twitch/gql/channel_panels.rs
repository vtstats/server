use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::gql_request;

static OPERATION: &str = "ChannelPanels";
static HASH: &str = "c388999b5fcd8063deafc7f7ad32ebd1cce3d94953c20bf96cffeef643327322";

pub async fn channel_panels(channel_id: &str, client: &Client) -> Result<Response> {
    let req = gql_request(
        client,
        OPERATION,
        Variables {
            id: channel_id.to_string(),
        },
        HASH,
    );

    let res = send_request!(req)?;

    let res: Response = res.json().await?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub user: User,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub login: String,
}
