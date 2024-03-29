use reqwest::Client;
use vtstats_utils::send_request;

use crate::youtubei::player::player;

pub async fn get_thumbnail(
    video_id: &str,
    client: &Client,
) -> anyhow::Result<(String, String, Vec<u8>)> {
    let response = player(video_id, client).await?;

    if let Some(url) = response.get_thumbnail_url() {
        let req = client.get(url);

        let res = send_request!(req, "/vi/:videoId.jpg")?;

        let (filename, content_type) = if url.contains("vi_webp") {
            (format!("{video_id}.webp"), "image/webp".into())
        } else {
            (format!("{video_id}.jpg"), "image/jpeg".into())
        };

        let bytes = res.bytes().await?.to_vec();

        return Ok((filename, content_type, bytes));
    }

    anyhow::bail!("Can't find thumbnail in player response, video_id={video_id}")
}
