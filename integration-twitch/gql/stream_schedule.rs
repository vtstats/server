use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::gql_request;

static OPERATION: &str = "StreamSchedule";
static HASH: &str = "d495cb17a67b6f7a8842e10297e57dcd553ea17fe691db435e39a618fe4699cf";

pub async fn stream_schedule(
    channel_username: String,
    start_at: DateTime<Utc>,
    client: &Client,
) -> Result<Response> {
    let req = gql_request(
        client,
        OPERATION.into(),
        Variables {
            login: channel_username,
            starting_weekday: "MONDAY".to_string(),
            utc_offset_minutes: 0,
            start_at,
            end_at: start_at + Duration::weeks(1),
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
    login: String,
    starting_weekday: String,
    utc_offset_minutes: usize,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
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
