use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub video_details: VideoDetails,
}

impl Response {
    pub fn get_thumbnail_url(&self) -> Option<&str> {
        self.video_details
            .thumbnail
            .thumbnails
            .iter()
            .max_by_key(|t| {
                // prefer webp format
                if t.url.contains("vi_webp") {
                    t.width + 1
                } else {
                    t.width
                }
            })
            .map(|t| t.url.as_str())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub title: String,
    pub channel_id: String,
    pub thumbnail: Thumbnail,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub thumbnails: Vec<ThumbnailUrl>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailUrl {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[test]
fn test() {
    let res = serde_json::from_str::<Response>(include_str!("./testdata/player.0.json")).unwrap();
    assert_eq!(
        res.get_thumbnail_url(),
        Some("https://i.ytimg.com/vi_webp/cQ_3OOspaPY/maxresdefault.webp?v=649d4b2e")
    );

    let res = serde_json::from_str::<Response>(include_str!("./testdata/player.1.json")).unwrap();
    assert_eq!(
        res.get_thumbnail_url(),
        Some("https://i.ytimg.com/vi/g2wDT7eMY-4/hqdefault.jpg?sqp=-oaymwEcCNACELwBSFXyq4qpAw4IARUAAIhCGAFwAcABBg==&rs=AOn4CLAUOIU-UTxNZSxWeKHgWgURJlYoWA")
    );
}
