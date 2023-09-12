use chrono::{Duration, DurationRound, Utc};
use integration_s3::upload_file;
use integration_youtube::{
    data_api::videos::list_videos,
    thumbnail::get_thumbnail,
    youtubei::{updated_metadata, updated_metadata_with_continuation},
};
use reqwest::Client;
use vtstats_database::{
    jobs::queue_send_notification,
    stream_stats::AddStreamViewerStatsQuery,
    streams::{
        delete_stream, end_stream, end_stream_with_values, start_stream, Stream, StreamStatus,
    },
    PgPool,
};

pub async fn collect_viewers(
    stream: &Stream,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut continuation: Option<String> = None;
    let mut status = stream.status;

    loop {
        let metadata = if let Some(continuation) = &continuation {
            updated_metadata_with_continuation(continuation, client).await
        } else {
            updated_metadata(&stream.platform_id, client).await
        }?;

        let (Some(timeout), Some(next_continuation)) =
            (metadata.timeout(), metadata.continuation())
        else {
            // stream not found
            if status == StreamStatus::Scheduled {
                delete_stream(stream.stream_id, pool).await?;
            } else {
                end_stream(stream.stream_id, pool).await?;
            }
            return Ok(());
        };

        continuation = Some(next_continuation.to_string());

        // stream is still waiting
        if metadata.is_waiting() {
            tokio::time::sleep(timeout).await;
            continue;
        }

        // record view stats
        if let Some(viewer) = metadata.view_count() {
            AddStreamViewerStatsQuery {
                time: Utc::now().duration_trunc(Duration::seconds(15))?,
                count: viewer,
                stream_id: stream.stream_id,
            }
            .execute(pool)
            .await?;
        }

        // stream has ended
        if timeout.as_secs() > 5 {
            if status == StreamStatus::Scheduled {
                delete_stream(stream.stream_id, pool).await?;
                return Ok(());
            }

            let mut videos = list_videos(&stream.platform_id, client).await?;
            let video: Option<integration_youtube::data_api::videos::Stream> =
                videos.pop().and_then(Into::into);

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
                video.as_ref().map(|s| s.title.as_str()),
                video.as_ref().and_then(|s| s.schedule_time),
                video.as_ref().and_then(|s| s.start_time),
                video.as_ref().and_then(|s| s.end_time),
                video.as_ref().and_then(|s| s.likes),
                thumbnail_url,
                pool,
            )
            .await?;

            queue_send_notification(
                Utc::now(),
                "youtube".into(),
                stream.platform_id.to_string(),
                stream.vtuber_id.to_string(),
                pool,
            )
            .await?;

            return Ok(());
        }

        if status == StreamStatus::Scheduled {
            start_stream(
                stream.stream_id,
                metadata.title().as_deref(),
                Utc::now(),
                metadata.like_count(),
                pool,
            )
            .await?;
            status = StreamStatus::Live;
        }

        tokio::time::sleep(timeout).await;
    }
}
