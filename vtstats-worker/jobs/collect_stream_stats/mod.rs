use chrono::Utc;

use integration_twitch::gql::channel_panels;
use reqwest::Client;
use tokio::signal::unix;
use vtstats_database::{
    channels::{get_channel_by_id, Platform},
    streams::{get_stream_by_id, StreamStatus},
    PgPool,
};

use super::JobResult;

pub mod twitch;
pub mod youtube;

pub async fn execute(pool: &PgPool, client: Client, stream_id: i32) -> anyhow::Result<JobResult> {
    let Some(stream) = get_stream_by_id(stream_id, pool).await? else {
        return Ok(JobResult::Completed);
    };

    if stream.status == StreamStatus::Ended {
        tracing::warn!("Stream {} is ended, skipping...", stream.stream_id);
        return Ok(JobResult::Completed);
    }

    let Some(channel) = get_channel_by_id(stream.channel_id, pool).await? else {
        return Ok(JobResult::Completed);
    };

    let (Ok(mut sigint), Ok(mut sigterm)) = (
        unix::signal(unix::SignalKind::interrupt()),
        unix::signal(unix::SignalKind::terminate()),
    ) else {
        anyhow::bail!("Failed to listen unix signal")
    };

    match stream.platform {
        Platform::Bilibili => {
            anyhow::bail!("We don't support bilibili stream")
        }
        Platform::Youtube => {
            tokio::select! {
                res = youtube::collect_viewers(&stream, &client, pool) => {
                    res.map(|_| JobResult::Completed)
                },
                res = youtube::collect_chats(&channel, &stream, &client, pool) => {
                    res.map(|_| JobResult::Completed)
                },
                _ = sigint.recv() => {
                    Ok(JobResult::Next { run: Utc::now() })
                },
                _ = sigterm.recv() => {
                    Ok(JobResult::Next { run: Utc::now() })
                },
            }
        }
        Platform::Twitch => {
            let res = channel_panels(&channel.platform_id, &client).await?;
            let channel_login = &res.data.user.login;

            tokio::select! {
                res = twitch::check_if_online(stream_id, pool) => {
                    res.map(|_| JobResult::Completed)
                },
                res = twitch::collect_chats(stream_id, channel_login, pool) => {
                    res.map(|_| JobResult::Completed)
                },
                res = twitch::collect_viewers(stream_id, channel_login, &client, pool) => {
                    res.map(|_| JobResult::Completed)
                },
                _ = sigint.recv() => {
                    Ok(JobResult::Next { run: Utc::now()  })
                },
                _ = sigterm.recv() => {
                    Ok(JobResult::Next { run: Utc::now()  })
                },
            }
        }
    }
}
