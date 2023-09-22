pub mod collect_channel_stats;
pub mod collect_stream_stats;
pub mod health_check;
pub mod refresh_youtube_rss;
pub mod send_notification;
pub mod subscribe_youtube_pubsub;

use chrono::{DateTime, Utc};
use metrics::{decrement_gauge, histogram, increment_gauge};
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
    let next_run = job.next_run;
    let job_type = payload.kind_str();
    let stream_id = match &payload {
        CollectYoutubeStreamMetadata(p) => Some(p.stream_id),
        CollectTwitchStreamMetadata(p) => Some(p.stream_id),
        SendNotification(p) => Some(p.stream_id),
        _ => None,
    };

    let span = match &payload {
        HealthCheck => tracing::trace_span!("Ignored"),
        _ => tracing::info_span!(
            "Worker Job",
            "message" = &format!("Job {job_type}"),
            "job_id" = job_id,
            "job_type" = job_type,
            "stream_id" = stream_id,
        ),
    };

    async move {
        let start = Instant::now();
        let last_run = Utc::now();

        increment_gauge!(
            "worker_jobs_running_count",
            1.,
            "kind" => job_type,
            "id" => job_id.to_string(),
        );

        let result = match payload {
            HealthCheck => health_check::execute().await,
            RefreshYoutubeRss => refresh_youtube_rss::execute(&pool, client).await,
            SubscribeYoutubePubsub => subscribe_youtube_pubsub::execute(&pool, client).await,
            UpdateChannelStats => collect_channel_stats::execute(&pool, &client).await,
            CollectYoutubeStreamMetadata(payload) => {
                collect_stream_stats::execute(&pool, client, payload.stream_id, next_run).await
            }
            CollectTwitchStreamMetadata(payload) => {
                collect_stream_stats::execute(&pool, client, payload.stream_id, next_run).await
            }
            SendNotification(payload) => {
                send_notification::execute(&pool, client, payload.stream_id).await
            }
        };

        let status = if result.is_ok() { "ok" } else { "err" };
        histogram!(
            "worker_jobs_elapsed_seconds",
            start.elapsed(),
            "kind" => job_type,
            "status" => status
        );
        decrement_gauge!(
            "worker_jobs_running_count",
            1.,
            "kind" => job_type,
            "id" => job_id.to_string(),
        );

        let query = match result {
            Ok(JobResult::Next { run }) => UpdateJobQuery {
                job_id,
                status: JobStatus::Queued,
                next_run: Some(run),
                last_run,
            },
            Ok(JobResult::Completed) => UpdateJobQuery {
                job_id,
                status: JobStatus::Success,
                next_run: None,
                last_run,
            },
            Err(ref err) => {
                tracing::error!(exception.stacktrace = ?err, message= %err);

                UpdateJobQuery {
                    job_id,
                    status: JobStatus::Failed,
                    next_run: None,
                    last_run,
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
