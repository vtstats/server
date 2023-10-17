use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::persisted_gql_request;

static OPERATION: &str = "VideoComments";
static HASH: &str = "f3b546321ec4632bcb83ee6a6dba91dad754fca3fd147ae26d9a7a0a096cfc60";

pub async fn video_comments(video_id: &str, client: &Client) -> Result<Response> {
    let req = persisted_gql_request(
        client,
        OPERATION,
        Variables {
            video_id: video_id.to_string(),
            has_video_id: true,
        },
        HASH,
    );

    let res = send_request!(req, "/gql/VideoComments")?;

    let res: Response = res.json().await?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variables {
    #[serde(rename = "videoID")]
    pub video_id: String,
    #[serde(rename = "hasVideoID")]
    pub has_video_id: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub cheer_config: CheerConfig,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheerConfig {
    pub groups: Vec<Group>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub nodes: Vec<Node>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub prefix: String,
}

#[test]
fn de() {
    serde_json::from_str::<Response>(include_str!("./testdata/video_comments.json")).unwrap();
}
