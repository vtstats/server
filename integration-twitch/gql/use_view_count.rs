use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::gql_request;

static OPERATION: &str = "UseViewCount";
static HASH: &str = "00b11c9c428f79ae228f30080a06ffd8226a1f068d6f52fbc057cbde66e994c2";

pub async fn use_view_count(channel_login: String, client: &Client) -> Result<Response> {
    let req = gql_request(client, OPERATION, Variables { channel_login }, HASH);

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
    pub stream: Option<Stream>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub viewers_count: i32,
}

#[test]
fn de() {
    use serde_json::from_str;

    from_str::<Response>(
        r#"{
            "data": {
              "user": {
                "id": "1117857350",
                "stream": null,
                "__typename": "User"
              }
            },
            "extensions": {
              "durationMilliseconds": 38,
              "operationName": "UseViewCount",
              "requestID": "01H9XC3B66VM13JGDCSQSJZBV6"
            }
          }"#,
    )
    .unwrap();

    from_str::<Response>(
        r#"{
            "data": {
              "user": {
                "id": "902183508",
                "stream": {
                  "id": "39941506117",
                  "viewersCount": 2855,
                  "__typename": "Stream"
                },
                "__typename": "User"
              }
            },
            "extensions": {
              "durationMilliseconds": 44,
              "operationName": "UseViewCount",
              "requestID": "01H9XC49J729SJ9GP8Q6F93SZ4"
            }
          }"#,
    )
    .unwrap();
}
