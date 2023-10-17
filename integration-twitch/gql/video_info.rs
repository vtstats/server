use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use vtstats_utils::send_request;

use super::gql_request;

static QUERY: &str = "query{video(id:$videoId){title,thumbnailURLs(height:180,width:320),previewThumbnailURL,createdAt,lengthSeconds}}";

pub async fn video_info(video_id: &str, client: &Client) -> Result<Response> {
    let req = gql_request(
        client,
        Variables {
            video_id: video_id.to_string(),
        },
        QUERY,
    );

    let res = send_request!(req, "/gql/video")?;

    let res: Response = res.json().await?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub video_id: String,
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
    pub login: String,
}
