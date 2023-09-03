use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use vtstats_database::streams::{Stream, StreamStatus};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub video_details: VideoDetails,
    #[serde(default)]
    pub microformat: Option<Microformat>,
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

    pub fn to_stream(&self) -> Option<Stream> {
        let format = self.microformat.clone()?.player_microformat_renderer;
        Some(Stream {
            title: format.title.simple_text,
            platform_id: self.video_details.video_id.clone(),
            status: StreamStatus::Ended,
            start_time: Some(format.live_broadcast_details.start_timestamp),
            end_time: Some(format.live_broadcast_details.end_timestamp),
            highlighted_title: None,
            schedule_time: None,
            like_max: None,
            updated_at: Utc::now(),
            stream_id: 0,
            thumbnail_url: None,
            vtuber_id: "".into(),
            viewer_avg: None,
            viewer_max: None,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub title: String,
    pub video_id: String,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    pub player_microformat_renderer: PlayerMicroformatRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMicroformatRenderer {
    pub title: Title,
    pub view_count: String,
    pub owner_channel_name: String,
    pub live_broadcast_details: LiveBroadcastDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBroadcastDetails {
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
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

    serde_json::from_str::<Response>(include_str!("./testdata/player.2.json")).unwrap();
}
