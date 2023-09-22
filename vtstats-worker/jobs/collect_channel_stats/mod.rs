mod bilibili_subscriber;
mod bilibili_view;
mod twitch_revenue;
mod twitch_subscriber;
mod youtube_revenue;
mod youtube_view_subscriber;

use chrono::{Duration, DurationRound, Utc};
use futures::TryFutureExt;
use reqwest::Client;
use tokio::join;

use vtstats_database::{
    channels::{list_active_channels_by_platform, Platform},
    PgPool,
};

use super::JobResult;

pub async fn execute(pool: &PgPool, client: &Client) -> anyhow::Result<JobResult> {
    let now = Utc::now().duration_trunc(Duration::hours(1))?;

    let youtube_channels = list_active_channels_by_platform(Platform::Youtube, pool).await?;
    let bilibili_channels = list_active_channels_by_platform(Platform::Bilibili, pool).await?;
    let twitch_channels = list_active_channels_by_platform(Platform::Twitch, pool).await?;

    let _ = join!(
        bilibili_subscriber::run(&bilibili_channels, client, now, pool).map_err(|err| {
            tracing::error!("Can't collect subscriber for bilibili: {err}");
        }),
        bilibili_view::run(&bilibili_channels, client, now, pool).map_err(|err| {
            tracing::error!("Can't collect view for bilibili: {err}");
        }),
        twitch_revenue::run(&twitch_channels, now, pool).map_err(|err| {
            tracing::error!("Can't collect revenue for twitch: {err}");
        }),
        twitch_subscriber::run(&twitch_channels, client, now, pool).map_err(|err| {
            tracing::error!("Can't collect subscriber for twitch: {err}");
        }),
        youtube_revenue::run(&youtube_channels, now, pool).map_err(|err| {
            tracing::error!("Can't collect revenue for youtube: {err}");
        }),
        youtube_view_subscriber::run(&youtube_channels, client, now, pool).map_err(|err| {
            tracing::error!("Can't collect view_subscriber for youtube: {err}");
        }),
    );

    Ok(JobResult::Next {
        run: now + Duration::hours(1),
    })
}
