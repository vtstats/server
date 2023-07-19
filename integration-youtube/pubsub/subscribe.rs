use anyhow::Result;
use reqwest::{header::CONTENT_TYPE, Client};
use std::env;

pub struct SubscribeYouTubePubsubQuery {
    pub channel_id: String,
}

impl SubscribeYouTubePubsubQuery {
    pub async fn send(&self, client: &Client) -> Result<()> {
        const TOPIC_BASE_URL: &str = "https://www.youtube.com/xml/feeds/videos.xml?channel_id=";

        let body = format!(
            "hub.callback=https://{}/api/pubsub&hub.topic={}{}&hub.mode=subscribe&hub.secret={}",
            env::var("SERVER_HOSTNAME")?,
            TOPIC_BASE_URL,
            self.channel_id,
            env::var("YOUTUBE_PUBSUB_SECRET")?
        );

        let req = client
            .post("https://pubsubhubbub.appspot.com/subscribe")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body);

        // let _res = crate::otel::send(&self.client, req).await?;

        req.send().await?;

        Ok(())
    }
}
