use chrono::{Duration, DurationRound, Utc};
use futures::{stream, TryStreamExt};
use reqwest::Client;

use integration_s3::upload_file;
use integration_youtube::{
    data_api::videos::{list_videos, Stream},
    rss_feed::FetchYouTubeVideosRSS,
    thumbnail::get_thumbnail,
};
use vtstats_database::{
    channels::{list_active_channels_by_platform, Platform},
    streams::{ListYouTubeStreamsQuery, UpsertStreamQuery},
    PgPool,
};

use super::JobResult;

pub async fn execute(pool: &PgPool, client: Client) -> anyhow::Result<JobResult> {
    let now = Utc::now().duration_trunc(Duration::hours(1)).unwrap();

    let now_str = now.to_string();

    let youtube_channels = list_active_channels_by_platform(Platform::Youtube, pool).await?;

    let feeds = stream::unfold(youtube_channels.iter(), |mut iter| async {
        let channel = iter.next()?;
        let res = FetchYouTubeVideosRSS {
            channel_id: channel.platform_id.to_string(),
            ts: now_str.clone(),
        }
        .execute(&client)
        .await;
        Some((res, iter))
    })
    .try_collect::<Vec<String>>()
    .await?;

    let video_ids = feeds
        .iter()
        .filter_map(|feed| find_video_id(feed))
        .collect::<Vec<_>>();

    let existed = ListYouTubeStreamsQuery {
        platform_ids: &video_ids,
        limit: None,
        ..Default::default()
    }
    .execute(pool)
    .await?;

    let missing = video_ids
        .into_iter()
        .filter(|id| existed.iter().all(|stream| &stream.platform_id != id))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(JobResult::Next {
            run: now + Duration::hours(1),
        });
    }

    tracing::debug!("Missing video ids: {:?}", missing);

    let mut streams: Vec<Stream> = Vec::with_capacity(missing.len());

    // youtube limits 50 streams per request
    for chunk in missing.chunks(50) {
        let videos = list_videos(&chunk.join(","), &client).await?;
        streams.extend(videos.into_iter().filter_map(Into::into));
    }

    if streams.is_empty() {
        tracing::warn!("Stream not found, ids={:?}", missing);
        return Ok(JobResult::Next {
            run: now + Duration::hours(1),
        });
    }

    for stream in streams {
        let channel = youtube_channels
            .iter()
            .find(|ch| ch.platform_id == stream.channel_id);

        let Some(channel) = channel else {
            continue;
        };

        let thumbnail_url = match get_thumbnail(&stream.id, &client).await {
            Ok((filename, content_type, bytes)) => {
                match upload_file(&filename, bytes, &content_type, &client).await {
                    Ok(thumbnail_url) => Some(thumbnail_url),
                    Err(err) => {
                        tracing::error!(exception.stacktrace = ?err, message= %err);
                        None
                    }
                }
            }
            Err(err) => {
                tracing::error!(exception.stacktrace = ?err, message= %err);
                None
            }
        };

        UpsertStreamQuery {
            platform: Platform::Youtube,
            vtuber_id: &channel.vtuber_id,
            platform_stream_id: &stream.id,
            channel_id: channel.channel_id,
            title: &stream.title,
            status: stream.status,
            thumbnail_url,
            schedule_time: stream.schedule_time,
            start_time: stream.start_time,
            end_time: stream.end_time,
        }
        .execute(pool)
        .await?;
    }

    Ok(JobResult::Next {
        run: now + Duration::hours(1),
    })
}

// TODO: add unit tests

fn find_video_id(feed: &str) -> Option<String> {
    // <yt:videoId>XXXXXXXXXXX</yt:videoId>
    Some(
        feed.lines()
            .nth(14)?
            .trim()
            .strip_prefix("<yt:videoId>")?
            .strip_suffix("</yt:videoId>")?
            .into(),
    )
}
