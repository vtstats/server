mod channel_avatar;
mod channel_panels;
mod stream_metadata;
mod stream_schedule;

pub use channel_avatar::channel_avatar;
pub use channel_panels::channel_panels;
pub use stream_metadata::stream_metadata;
pub use stream_schedule::stream_schedule;

use reqwest::{Client, RequestBuilder};
use serde::Serialize;

static CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

pub fn gql_request<V: Serialize>(
    client: &Client,
    operation_name: String,
    variables: V,
    hash: String,
) -> RequestBuilder {
    client
        .post("https://gql.twitch.tv/gql")
        .header("Client-Id", CLIENT_ID)
        .json(&Request {
            operation_name,
            variables,
            extensions: Extensions {
                persisted_query: PersistedQuery {
                    version: 1,
                    sha256hash: hash,
                },
            },
        })
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<V: Serialize> {
    pub operation_name: String,
    pub variables: V,
    pub extensions: Extensions,
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
    pub sha256hash: String,
}
