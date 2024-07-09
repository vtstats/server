use chrono::{DateTime, Utc};

use integration_twitch::gql::channel_panels;
use tokio::signal::unix;
use vtstats_database::{
    channels::{get_channel_by_id, Platform},
    streams::{get_stream_by_id, StreamStatus},
};
use vtstats_utils::context::AppContext;

use super::JobResult;

pub mod twitch;
pub mod youtube;

pub async fn execute(
    stream_id: i32,
    ctx: &AppContext,
    next_run: Option<DateTime<Utc>>,
) -> anyhow::Result<JobResult> {
    let Some(stream) = get_stream_by_id(stream_id, &ctx.pool).await? else {
        return Ok(JobResult::Completed);
    };

    if stream.status == StreamStatus::Ended {
        tracing::warn!("Stream {} is ended, skipping...", stream.stream_id);
        return Ok(JobResult::Completed);
    }

    let Some(channel) = get_channel_by_id(stream.channel_id, &ctx.pool).await? else {
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
                res = youtube::collect_viewers(&stream, &ctx) => {
                    res.map(|_| JobResult::Completed)
                },
                res = youtube::collect_chats(&channel, &stream, &ctx.client, &ctx.pool) => {
                    res.map(|_| JobResult::Completed)
                },
                _ = sigint.recv() => {
                    let run = next_run.unwrap_or_else(Utc::now);
                    Ok(JobResult::Next { run })
                },
                _ = sigterm.recv() => {
                    let run = next_run.unwrap_or_else(Utc::now);
                    Ok(JobResult::Next { run })
                },
            }
        }
        Platform::Twitch => {
            let res = channel_panels(&channel.platform_id, &ctx.client).await?;
            let channel_login = &res.data.user.login;

            tokio::select! {
                res = twitch::check_if_online(stream_id, &ctx.pool) => {
                    res.map(|_| JobResult::Completed)
                },
                res = twitch::collect_chats(stream_id, channel_login, &ctx.pool) => {
                    res.map(|_| JobResult::Completed)
                },
                res = twitch::collect_viewers(stream_id, channel_login, &ctx.client, &ctx.pool) => {
                    res.map(|_| JobResult::Completed)
                },
                _ = sigint.recv() => {
                    let run = next_run.unwrap_or_else(Utc::now);
                    Ok(JobResult::Next { run })
                },
                _ = sigterm.recv() => {
                    let run = next_run.unwrap_or_else(Utc::now);
                    Ok(JobResult::Next { run })
                },
            }
        }
    }
}
