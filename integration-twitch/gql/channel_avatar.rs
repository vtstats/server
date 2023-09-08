use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::gql_request;

static OPERATION: &str = "ChannelAvatar";
static HASH: &str = "84ed918aaa9aaf930e58ac81733f552abeef8ac26c0117746865428a7e5c8ab0";

pub async fn channel_avatar(channel_username: String, client: &Client) -> Result<Response> {
    let req = gql_request(
        client,
        OPERATION.into(),
        Variables {
            channel_login: channel_username,
        },
        HASH.into(),
    );

    let res = send_request!(req)?;

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
    use serde_json::from_str;

    from_str::<Response>(
        r#"{
            "data": {
                "user": {
                "id": "583341489",
                "followers": {
                    "totalCount": 70085,
                    "__typename": "FollowerConnection"
                },
                "isPartner": true,
                "primaryColorHex": null,
                "__typename": "User"
                }
            },
            "extensions": {
                "durationMilliseconds": 42,
                "operationName": "ChannelAvatar",
                "requestID": "01H9NBWDNRAJMTYDQDVVYYY39A"
            }
        }"#,
    )
    .unwrap();
}
