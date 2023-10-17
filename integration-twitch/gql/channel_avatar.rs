use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::persisted_gql_request;

static OPERATION: &str = "ChannelAvatar";
static HASH: &str = "84ed918aaa9aaf930e58ac81733f552abeef8ac26c0117746865428a7e5c8ab0";

pub async fn channel_avatar(channel_login: String, client: &Client) -> Result<Response> {
    let req = persisted_gql_request(client, OPERATION, Variables { channel_login }, HASH);

    let res = send_request!(req, "/gql/ChannelAvatar")?;

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
    pub followers: Followers,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Followers {
    pub total_count: i32,
}

#[test]
fn de() {
    serde_json::from_str::<Response>(include_str!("./testdata/channel_avatar.json")).unwrap();
}
