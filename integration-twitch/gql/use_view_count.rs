use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::persisted_gql_request;

static OPERATION: &str = "UseViewCount";
static HASH: &str = "00b11c9c428f79ae228f30080a06ffd8226a1f068d6f52fbc057cbde66e994c2";

pub async fn use_view_count(channel_login: String, client: &Client) -> Result<Response> {
    let req = persisted_gql_request(client, OPERATION, Variables { channel_login }, HASH);

    let res = send_request!(req, "/gql/UseViewCount")?;

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
    pub stream: Option<Stream>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub viewers_count: i32,
}

#[test]
fn de() {
    serde_json::from_str::<Response>(include_str!("./testdata/user_view_count.0.json")).unwrap();
    serde_json::from_str::<Response>(include_str!("./testdata/user_view_count.1.json")).unwrap();
}
