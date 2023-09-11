pub mod collect_stream_stats;
pub mod health_check;
pub mod refresh_youtube_rss;
pub mod send_notification;
pub mod subscribe_youtube_pubsub;
pub mod update_channel_stats;

use chrono::{DateTime, Utc};
use metrics::{histogram, increment_counter};
use reqwest::Client;
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use tracing::Instrument;

use vtstats_database::{
    jobs::{Job, JobPayload::*, JobStatus, UpdateJobQuery},
    PgPool,
};

pub enum JobResult {
    Completed,
    Next { run: DateTime<Utc> },
}

pub async fn execute(job: Job, pool: PgPool, client: Client, _shutdown_complete_tx: Sender<()>) {
    let job_id = job.job_id;
    let payload = job.payload;
    let job_type = payload.kind_str();

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
            RefreshYoutubeRss => refresh_youtube_rss::execute(&pool, client).await,
            SubscribeYoutubePubsub => subscribe_youtube_pubsub::execute(&pool, client).await,
            UpdateChannelStats => update_channel_stats::execute(&pool, &client).await,
            CollectYoutubeStreamMetadata(payload) => {
                collect_stream_stats::execute(&pool, client, payload.stream_id).await
            }
            CollectTwitchStreamMetadata(payload) => {
                collect_stream_stats::execute(&pool, client, payload.stream_id).await
            }
            SendNotification(payload) => send_notification::execute(&pool, client, payload).await,
            // TODO: remove
            UpsertYoutubeStream(_) => Ok(JobResult::Completed),
            UpdateCurrencyExchangeRate => Ok(JobResult::Completed),
            InstallDiscordCommands => Ok(JobResult::Completed),
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
            Ok(JobResult::Next { run }) => UpdateJobQuery {
                job_id,
                status: JobStatus::Queued,
                next_run: Some(run),
                last_run: Utc::now(),
            },
            Ok(JobResult::Completed) => UpdateJobQuery {
                job_id,
                status: JobStatus::Success,
                next_run: None,
                last_run: Utc::now(),
            },
            Err(ref err) => {
                tracing::error!(exception.stacktrace = ?err, message= %err);

                UpdateJobQuery {
                    job_id,
                    status: JobStatus::Failed,
                    next_run: None,
                    last_run: Utc::now(),
                }
            }
        };

        if let Err(err) = query.execute(&pool).await {
            tracing::error!("[Database Error] {err:?}");
        }
    }
    .instrument(span)
    .await;
}
