mod channel_avatar;
mod channel_panels;
mod stream_metadata;
mod stream_schedule;
mod use_view_count;
mod video_comments;
pub mod video_comments_by_offset_or_cursor;
mod video_info;

pub use channel_avatar::channel_avatar;
pub use channel_panels::channel_panels;
pub use stream_metadata::stream_metadata;
pub use stream_schedule::stream_schedule;
pub use use_view_count::use_view_count;
pub use video_comments::video_comments;
pub use video_comments_by_offset_or_cursor::video_comments_by_offset_or_cursor;
pub use video_info::video_info;

use reqwest::{Client, RequestBuilder};
use serde::Serialize;
use serde_with::skip_serializing_none;

static CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

pub fn persisted_gql_request<V: Serialize>(
    client: &Client,
    operation_name: &'static str,
    variables: V,
    hash: &'static str,
) -> RequestBuilder {
    client
        .post("https://gql.twitch.tv/gql")
        .header("Client-Id", CLIENT_ID)
        .json(&Request {
            operation_name: Some(operation_name),
            variables,
            query: None,
            extensions: Some(Extensions {
                persisted_query: PersistedQuery {
                    version: 1,
                    sha256hash: hash,
                },
            }),
        })
}

pub fn persisted_gql_request_with_integrity<V: Serialize>(
    client: &Client,
    operation_name: &'static str,
    variables: V,
    hash: &'static str,
    integrity: &str,
    device_id: &str,
) -> RequestBuilder {
    client
        .post("https://gql.twitch.tv/gql")
        .header("Client-Id", CLIENT_ID)
        .header("Client-Integrity", integrity)
        .header("X-Device-Id", device_id)
        .json(&Request {
            operation_name: Some(operation_name),
            variables,
            query: None,
            extensions: Some(Extensions {
                persisted_query: PersistedQuery {
                    version: 1,
                    sha256hash: hash,
                },
            }),
        })
}

pub fn gql_request<V: Serialize>(
    client: &Client,
    variables: V,
    query: &'static str,
) -> RequestBuilder {
    client
        .post("https://gql.twitch.tv/gql")
        .header("Client-Id", CLIENT_ID)
        .json(&Request {
            variables,
            query: Some(query),
            operation_name: None,
            extensions: None,
        })
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<V: Serialize> {
    pub operation_name: Option<&'static str>,
    pub variables: V,
    pub query: Option<&'static str>,
    pub extensions: Option<Extensions>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    pub persisted_query: PersistedQuery,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedQuery {
    pub version: i64,
    #[serde(rename = "sha256Hash")]
    pub sha256hash: &'static str,
}
