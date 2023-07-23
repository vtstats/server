use anyhow::Result;
use bytes::Bytes;
use futures::TryFutureExt;
use tracing::instrument;

use super::RequestHub;
use integration_s3::upload_file;
use vtstat_utils::instrument_send;

impl RequestHub {
    #[instrument(name = "Upload thumbnail", skip(self))]
    pub async fn upload_thumbnail(&self, stream_id: &str) -> Option<String> {
        let data = match self.youtube_thumbnail(stream_id).await {
            Ok(x) => x,
            Err(err) => {
                tracing::error!("Failed to upload thumbnail: {:?}", err);
                return None;
            }
        };

        let filename = format!("{}.webp", stream_id);

        match upload_file(&filename, data, "image/webp", &self.client).await {
            Ok(url) => Some(url),
            Err(err) => {
                tracing::error!("Failed to upload thumbnail: {:?}", err);
                None
            }
        }
    }

    async fn youtube_thumbnail(&self, id: &str) -> Result<Bytes> {
        self.youtube_thumbnail_by_res(id, "maxresdefault")
            .or_else(|_| self.youtube_thumbnail_by_res(id, "sddefault"))
            .or_else(|_| self.youtube_thumbnail_by_res(id, "mqdefault"))
            .or_else(|_| self.youtube_thumbnail_by_res(id, "hqdefault"))
            .await
    }

    async fn youtube_thumbnail_by_res(&self, id: &str, res: &str) -> Result<Bytes> {
        let url = format!("https://i.ytimg.com/vi_webp/{}/{}.webp", id, res);

        let req = self.client.get(url);

        let res = instrument_send(&self.client, req).await?;

        let bytes = res.bytes().await?;

        Ok(bytes)
    }
}
