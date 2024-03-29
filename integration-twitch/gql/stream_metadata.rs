use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::persisted_gql_request;

static OPERATION: &str = "StreamMetadata";
static HASH: &str = "252a46e3f5b1ddc431b396e688331d8d020daec27079893ac7d4e6db759a7402";

pub async fn stream_metadata(channel_login: &str, client: &Client) -> Result<Response> {
    let req = persisted_gql_request(
        client,
        OPERATION,
        Variables {
            channel_login: channel_login.to_string(),
        },
        HASH,
    );

    let res = send_request!(req, "/gql/StreamMetadata")?;

    let res: Response = res.json().await?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub channel_login: String,
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
    pub last_broadcast: LastBroadcast,
    pub stream: Option<Stream>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastBroadcast {
    pub id: Option<String>,
    pub title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub created_at: String,
}

#[test]
fn de() {
    serde_json::from_str::<Response>(include_str!("./testdata/stream_metadata.0.json")).unwrap();
    serde_json::from_str::<Response>(include_str!("./testdata/stream_metadata.1.json")).unwrap();
}
