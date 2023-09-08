use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::gql_request;

static OPERATION: &str = "StreamMetadata";
static HASH: &str = "252a46e3f5b1ddc431b396e688331d8d020daec27079893ac7d4e6db759a7402";

pub async fn stream_metadata(channel_login: &str, client: &Client) -> Result<Response> {
    let req = gql_request(
        client,
        OPERATION.into(),
        Variables {
            channel_login: channel_login.to_string(),
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
    serde_json::from_str::<Response>(
        r#"
    {
        "data": {
            "user": {
                "id": "151054406",
                "primaryColorHex": "00FA04",
                "isPartner": true,
                "profileImageURL": "https://static-cdn.jtvnw.net/jtv_user_pictures/4e39c083-2a9a-46dc-95ad-6b74bff0c34b-profile_image-70x70.png",
                "primaryTeam": {
                    "id": "10123",
                    "name": "vshojo",
                    "displayName": "VShojo",
                    "__typename": "Team"
                },
                "squadStream": null,
                "channel": {
                    "id": "151054406",
                    "chanlets": null,
                    "__typename": "Channel"
                },
                "lastBroadcast": {
                    "id": "39912836949",
                    "title": "🐸 space frogs! 💀 !discord !social",
                    "__typename": "Broadcast"
                },
                "stream": null,
                "__typename": "User"
            }
        },
        "extensions": {
            "durationMilliseconds": 43,
            "operationName": "StreamMetadata",
            "requestID": "01H9SW1NW6D00HM2B8TXMC3MY1"
        }
    }
    "#,
    ).unwrap();

    serde_json::from_str::<Response>(
        r#"{
            "data": {
                "user": {
                    "id": "883696964",
                    "primaryColorHex": null,
                    "isPartner": false,
                    "profileImageURL": "https://static-cdn.jtvnw.net/jtv_user_pictures/f6b07858-0629-4710-88fa-b404c695f49c-profile_image-70x70.png",
                    "primaryTeam": {
                        "id": "14086",
                        "name": "nijisanji",
                        "displayName": "にじさんじ",
                        "__typename": "Team"
                    },
                    "squadStream": null,
                    "channel": {
                        "id": "883696964",
                        "chanlets": null,
                        "__typename": "Channel"
                    },
                    "lastBroadcast": {
                        "id": "40595948983",
                        "title": "VALORANT復活！💎１～",
                        "__typename": "Broadcast"
                    },
                    "stream": {
                        "id": "40595948983",
                        "type": "live",
                        "createdAt": "2023-09-08T05:59:46Z",
                        "game": {
                            "id": "516575",
                            "slug": "valorant",
                            "name": "VALORANT",
                            "__typename": "Game"
                        },
                        "__typename": "Stream"
                    },
                    "__typename": "User"
                }
            },
            "extensions": {
                "durationMilliseconds": 43,
                "operationName": "StreamMetadata",
                "requestID": "01H9SW4ZJTC6F25K1ZMKFVD6BT"
            }
        }"#,
    ).unwrap();
}
