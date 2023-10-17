use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::persisted_gql_request;

static OPERATION: &str = "StreamSchedule";
static HASH: &str = "d495cb17a67b6f7a8842e10297e57dcd553ea17fe691db435e39a618fe4699cf";

pub async fn stream_schedule(
    channel_login: String,
    start_at: DateTime<Utc>,
    client: &Client,
) -> Result<Response> {
    let req = persisted_gql_request(
        client,
        OPERATION,
        Variables {
            login: channel_login,
            starting_weekday: "MONDAY".to_string(),
            utc_offset_minutes: 0,
            start_at,
            end_at: start_at + Duration::weeks(1),
        },
        HASH,
    );

    let res = send_request!(req, "/gql/StreamSchedule")?;

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
    pub videos: Videos,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Videos {
    pub edges: Vec<Edge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub node: Node,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub title: String,
    pub created_at: String,
}

#[test]
fn de() {
    serde_json::from_str::<Response>(include_str!("./testdata/stream_schedule.json")).unwrap();
}
