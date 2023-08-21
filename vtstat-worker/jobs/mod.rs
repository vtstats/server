pub mod collect_youtube_stream_metadata;
pub mod health_check;
pub mod install_discord_command;
pub mod refresh_youtube_rss;
pub mod send_notification;
pub mod subscribe_youtube_pubsub;
pub mod update_channel_stats;
pub mod update_currency_exchange_rate;
pub mod upsert_youtube_stream;

use chrono::{DateTime, Utc};
use metrics::{histogram, increment_counter};
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use tracing::Instrument;

use vtstat_database::{
    jobs::{Job, JobPayload::*, JobStatus, UpdateJobQuery},
    PgPool,
};
use vtstat_request::RequestHub;

pub enum JobResult {
    Completed,
    Next {
        run: DateTime<Utc>,
        continuation: Option<String>,
    },
}

pub async fn execute(job: Job, pool: PgPool, hub: RequestHub, _shutdown_complete_tx: Sender<()>) {
    let job_id = job.job_id;
    let payload = job.payload;
    let continuation = job.continuation;

    let job_type = match &payload {
        HealthCheck => "health_check",
        RefreshYoutubeRss => "refresh_youtube_rss",
        SubscribeYoutubePubsub => "subscribe_youtube_pubsub",
        UpdateChannelStats => "update_channel_stats",
        UpdateCurrencyExchangeRate => "update_currency_exchange_rate",
        UpsertYoutubeStream(_) => "upsert_youtube_stream",
        CollectYoutubeStreamMetadata(_) => "collect_youtube_stream_metadata",
        SendNotification(_) => "send_notification",
        InstallDiscordCommands => "install_discord_commands",
    };

    let span = match &payload {
        HealthCheck => tracing::trace_span!("Ignored"),
        _ => tracing::info_span!(
            "Worker Job",
            "message" = &format!("Job {job_type}"),
            "job.id" = job_id,
            "job.type" = job_type,
        ),
    };

    async move {
        let start = Instant::now();

        let result = match payload {
            HealthCheck => health_check::execute().await,
            RefreshYoutubeRss => refresh_youtube_rss::execute(&pool, hub).await,
            SubscribeYoutubePubsub => subscribe_youtube_pubsub::execute(&pool, hub).await,
            UpdateChannelStats => update_channel_stats::execute(&pool, &hub.client).await,
            // TODO:
            UpdateCurrencyExchangeRate => update_currency_exchange_rate::execute(&pool).await,
            UpsertYoutubeStream(payload) => {
                upsert_youtube_stream::execute(&pool, hub, payload).await
            }
            CollectYoutubeStreamMetadata(payload) => {
                collect_youtube_stream_metadata::execute(&pool, hub, continuation, payload).await
            }
            SendNotification(payload) => send_notification::execute(&pool, hub, payload).await,
            InstallDiscordCommands => install_discord_command::execute(&hub.client).await,
        };

        let status = if result.is_ok() { "ok" } else { "err" };
        histogram!(
            "worker_jobs_elapsed_seconds",
            start.elapsed(),
            "kind" => job_type,
            "status" => status
        );
        increment_counter!(
            "worker_jobs_count",
            "kind" => job_type,
            "status" => status
        );

        let query = match result {
            Ok(JobResult::Next { run, continuation }) => UpdateJobQuery {
                job_id,
                status: JobStatus::Queued,
                next_run: Some(run),
                last_run: Utc::now(),
                continuation,
            },
            Ok(JobResult::Completed) => UpdateJobQuery {
                job_id,
                status: JobStatus::Success,
                next_run: None,
                last_run: Utc::now(),
                continuation: None,
            },
            Err(ref err) => {
                tracing::error!(exception.stacktrace = ?err, message= %err);

                UpdateJobQuery {
                    job_id,
                    status: JobStatus::Failed,
                    next_run: None,
                    last_run: Utc::now(),
                    continuation: None,
                }
            }
        };

        if let Err(err) = query.execute(&pool).await {
            eprintln!("[Database Error] {err:?}");
        }
    }
    .instrument(span)
    .await;
}
