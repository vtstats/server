use std::cmp;

use chrono::{Duration, DurationRound, Utc};
use reqwest::Client;

use integration_s3::upload_file;
use integration_youtube::{
    data_api::videos::list_videos,
    thumbnail::get_thumbnail,
    youtubei::{metadata::Response, updated_metadata, updated_metadata_with_continuation},
};
use vtstats_database::{
    jobs::queue_send_notification,
    stream_stats::AddStreamViewerStatsQuery,
    streams::{
        delete_stream, end_stream_with_values, get_stream_by_id, start_stream, Stream, StreamStatus,
    },
    PgPool,
};
use vtstats_utils::context::AppContext;

pub async fn collect_viewers(stream: &Stream, ctx: &AppContext) -> anyhow::Result<()> {
    let st = get_stream_by_id(stream.stream_id, &ctx.pool).await?;

    let mut last_max = st.as_ref().and_then(|st| st.viewer_max).unwrap_or_default();
    let mut last_avg = st.as_ref().and_then(|st| st.viewer_avg).unwrap_or_default();
    let start = st
        .as_ref()
        .and_then(|st| st.start_time)
        .unwrap_or_else(|| Utc::now().duration_trunc(Duration::seconds(15)).unwrap());

    let mut continuation: Option<String> = None;
    let mut status = stream.status;

    loop {
        let response = if let Some(continuation) = &continuation {
            updated_metadata_with_continuation(continuation, &ctx.client).await
        } else {
            updated_metadata(&stream.platform_id, &ctx.client).await
        }?;

        let bytes = response.bytes().await?;

        let metadata: Response = serde_json::from_slice(&bytes)?;

        let (Some(timeout), Some(next_continuation)) =
            (metadata.timeout(), metadata.continuation())
        else {
            // stream not found
            if status == StreamStatus::Scheduled {
                tracing::warn!("delete schedule stream {}", stream.platform_id);
                delete_stream(stream.stream_id, &ctx.pool, &ctx.search).await?;

                return Ok(());
            } else {
                let mut videos = list_videos(&stream.platform_id, &ctx.client).await?;
                let video: Option<integration_youtube::data_api::videos::Stream> =
                    videos.pop().and_then(Into::into);

                match video {
                    Some(video) if video.end_time.is_some() => {
                        tracing::warn!(
                            stream_id = stream.stream_id,
                            "delete schedule stream, platform_id={}",
                            stream.platform_id
                        );
                        end_youtube_stream(stream, video, &ctx.client, &ctx.pool).await?;
                        return Ok(());
                    }
                    _ => {
                        tracing::warn!(
                            stream_id = stream.stream_id,
                            "stream timeout and next_continuation is not found, but stream is not ended, platform_id={}",
                            stream.platform_id
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                }
            }
        };

        continuation = Some(next_continuation.to_string());

        // stream is still waiting
        if metadata.is_waiting() {
            tokio::time::sleep(timeout).await;
            continue;
        }

        // record view stats
        if let Some(viewer) = metadata.view_count() {
            let time = Utc::now().duration_trunc(Duration::seconds(15))?;

            let count = ((time - start).num_seconds() / 15) as i32;

            let max = cmp::max(last_max, viewer);
            let avg = (last_avg * count + viewer) / (count + 1);

            AddStreamViewerStatsQuery {
                time,
                count: viewer,
                stream_id: stream.stream_id,
                max,
                avg,
            }
            .execute(&ctx.pool)
            .await?;

            vtstats_database::streams::meilisearch::add_or_update(
                vtstats_database::streams::meilisearch::Document {
                    stream_id: stream.stream_id,
                    updated_at: time,
                    viewer_max: Some(max),
                    viewer_avg: Some(avg),
                    ..Default::default()
                },
                &ctx.search,
            )
            .await?;

            (last_max, last_avg) = (max, avg);
        }

        // stream has ended
        if timeout.as_secs() > 5 {
            if status == StreamStatus::Scheduled {
                tracing::warn!(
                    stream_id = stream.stream_id,
                    "delete schedule stream, platform_id={}",
                    stream.platform_id
                );
                delete_stream(stream.stream_id, &ctx.pool, &ctx.search).await?;
                return Ok(());
            } else {
                let mut videos = list_videos(&stream.platform_id, &ctx.client).await?;
                let video: Option<integration_youtube::data_api::videos::Stream> =
                    videos.pop().and_then(Into::into);

                match video {
                    Some(video) if video.end_time.is_some() => {
                        end_youtube_stream(stream, video, &ctx.client, &ctx.pool).await?;
                        return Ok(());
                    }
                    _ => {
                        tracing::warn!(
                            stream_id = stream.stream_id,
                            "stream timeout is {}ms, but stream is not ended, platform_id={}",
                            timeout.as_millis(),
                            stream.platform_id
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                }
            }
        }

        if status == StreamStatus::Scheduled {
            start_stream(
                stream.stream_id,
                metadata.title().as_deref(),
                Utc::now(),
                metadata.like_count(),
                &ctx.pool,
                &ctx.search,
            )
            .await?;
            status = StreamStatus::Live;
        }

        tokio::time::sleep(timeout).await;
    }
}

async fn end_youtube_stream(
    stream: &Stream,
    video: integration_youtube::data_api::videos::Stream,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    // update thumbnail url after stream is ended
    let thumbnail_url = match get_thumbnail(&stream.platform_id, client).await {
        Ok((filename, content_type, bytes)) => {
            match upload_file(&filename, bytes, &content_type, client).await {
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

    end_stream_with_values(
        stream.stream_id,
        Some(video.title.as_str()),
        video.schedule_time,
        video.start_time,
        video.end_time,
        video.likes,
        thumbnail_url,
        pool,
    )
    .await?;

    queue_send_notification(Utc::now(), stream.stream_id, pool).await?;

    Ok(())
}
