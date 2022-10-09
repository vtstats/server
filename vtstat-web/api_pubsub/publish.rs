use hmac::{Hmac, Mac};
use roxmltree::Document;
use sha1::Sha1;
use std::{convert::Into, env};
use tracing::Span;
use warp::{http::StatusCode, Rejection};

use vtstat_database::{
    channels::ListChannelsQuery,
    jobs::{JobPayload, PushJobQuery, UpsertYoutubeStreamJobPayload},
    streams::{EndStreamQuery, ListYouTubeStreamsQuery},
    PgPool,
};

use crate::reject::WarpError;

pub async fn publish_content(
    body: String,
    signature: String,
    pool: PgPool,
) -> Result<StatusCode, Rejection> {
    Span::current().record("name", &"POST /api/pubsub");

    tracing::debug!("body={}", body.as_str());

    let expected = generate_signature(&body)?;
    let found = signature.trim_start_matches("sha1=");

    if expected != found {
        tracing::error!("Bad signature, expected={}, found={}", expected, found);
        return Ok(StatusCode::BAD_REQUEST);
    }

    let doc = match Document::parse(&body) {
        Ok(doc) => doc,
        Err(err) => {
            tracing::error!("failed to parse xml: {:?}", err);
            return Ok(StatusCode::BAD_REQUEST);
        }
    };

    if let Some((platform_channel_id, platform_stream_id)) = parse_modification(&doc) {
        let channels = ListChannelsQuery {
            platform: "youtube",
        }
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

        let channel = channels
            .iter()
            .find(|ch| ch.platform_id == platform_channel_id);

        let Some(channel) = channel else {
            return Ok(StatusCode::NOT_FOUND);
        };

        PushJobQuery {
            continuation: None,
            next_run: None,
            payload: JobPayload::UpsertYoutubeStream(UpsertYoutubeStreamJobPayload {
                channel_id: channel.channel_id,
                platform_stream_id: platform_stream_id.into(),
            }),
        }
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

        return Ok(StatusCode::OK);
    }

    if let Some((_, platform_stream_id)) = parse_deletion(&doc) {
        let streams = ListYouTubeStreamsQuery {
            platform_ids: &[platform_stream_id.into()],
            ..Default::default()
        }
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

        let stream = streams
            .iter()
            .find(|stream| stream.platform_id == platform_stream_id);

        let Some(stream) = stream else {
            return Ok(StatusCode::NOT_FOUND);
        };

        EndStreamQuery {
            stream_id: stream.stream_id,
            ..Default::default()
        }
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

        return Ok(StatusCode::OK);
    }

    tracing::error!("Unknown xml schema");

    Ok(StatusCode::BAD_REQUEST)
}

pub fn generate_signature(data: &str) -> Result<String, Rejection> {
    let key = env::var("YOUTUBE_PUBSUB_SECRET").map_err(Into::<WarpError>::into)?;

    let mut mac =
        Hmac::<Sha1>::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(data.as_bytes());

    let result = mac.finalize().into_bytes();

    Ok(hex::encode(result))
}

pub fn parse_modification<'a>(doc: &'a Document) -> Option<(&'a str, &'a str)> {
    let stream_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "videoId")
        .and_then(|n| n.text())?;

    let channel_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "channelId")
        .and_then(|n| n.text())?;

    Some((channel_id, stream_id))
}

pub fn parse_deletion<'a>(doc: &'a Document) -> Option<(&'a str, &'a str)> {
    let stream_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "deleted-entry")
        .and_then(|n| n.attribute("ref"))
        .and_then(|r| r.get("yt:video:".len()..))?;

    let channel_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "uri")
        .and_then(|n| n.text())
        .and_then(|n| n.get("https://www.youtube.com/channel/".len()..))?;

    Some((channel_id, stream_id))
}
