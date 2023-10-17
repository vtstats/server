use std::borrow::Cow;

use chrono::{DateTime, Utc};
use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::persisted_gql_request_with_integrity;

static OPERATION: &str = "VideoCommentsByOffsetOrCursor";
static HASH: &str = "b70a3591ff0f4e0313d126c6a1502d79a1c02baebb288227c582044aa76adf6a";

pub async fn video_comments_by_offset_or_cursor(
    video_id: &str,
    cursor: Option<String>,
    client: &Client,
    integrity: &str,
    device_id: &str,
) -> Result<reqwest::Response> {
    let req = persisted_gql_request_with_integrity(
        client,
        OPERATION,
        Variables { video_id, cursor },
        HASH,
        integrity,
        device_id,
    );

    let res = send_request!(req, "/gql/VideoCommentsByOffsetOrCursor")?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variables<'v> {
    #[serde(rename = "videoID")]
    pub video_id: &'v str,
    pub cursor: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<'a> {
    pub data: Data<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data<'a> {
    pub video: Video<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video<'a> {
    pub comments: Comments<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comments<'a> {
    pub edges: Vec<Edge<'a>>,
    pub page_info: PageInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge<'a> {
    pub cursor: Option<Cow<'a, str>>,
    pub node: Node<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node<'a> {
    pub commenter: Commenter<'a>,
    pub content_offset_seconds: i64,
    pub created_at: DateTime<Utc>,
    pub message: Message<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commenter<'a> {
    pub id: Cow<'a, str>,
    pub login: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message<'a> {
    pub fragments: Vec<Fragment<'a>>,
    pub user_badges: Vec<UserBadge<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fragment<'a> {
    pub text: Cow<'a, str>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserBadge<'a> {
    pub id: Cow<'a, str>,
    #[serde(rename = "setID")]
    pub set_id: Cow<'a, str>,
    pub version: Cow<'a, str>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[test]
fn de() {
    serde_json::from_str::<Response>(include_str!(
        "./testdata/video_comments_by_offset_or_cursor.json"
    ))
    .unwrap();
}
