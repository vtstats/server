use reqwest::{Client, Url};
use roxmltree::Document;
use vtstats_utils::send_request;

pub struct FetchYouTubeVideosRSS {
    pub channel_id: String,
    // timestamp, used for cache busting
    pub ts: String,
}

impl FetchYouTubeVideosRSS {
    pub async fn execute(&self, client: &Client) -> anyhow::Result<String> {
        let url = Url::parse_with_params(
            "https://youtube.com/feeds/videos.xml",
            &[("channel_id", &self.channel_id), ("ts", &self.ts)],
        )?;

        let req = client
            .get(url)
            .header(reqwest::header::CACHE_CONTROL, "no-cache");

        let res = send_request!(req)?;

        let text = res.text().await?;

        find_first_video_id(&text)
    }
}

fn find_first_video_id(feed: &str) -> anyhow::Result<String> {
    let doc = Document::parse(feed)?;

    let id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "videoId")
        .and_then(|n| n.text())
        .ok_or_else(|| anyhow::anyhow!("Cannot find videoId"))?;

    Ok(id.into())
}

#[test]
fn parse_xml() {
    assert_eq!(
        find_first_video_id(include_str!("./testdata/feed.0.xml"),).unwrap(),
        "HxK06JNLqAk".to_string()
    );
}
