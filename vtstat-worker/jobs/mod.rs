mod collect_youtube_stream_live_chat;
mod collect_youtube_stream_metadata;
mod health_check;
mod refresh_youtube_rss;
mod subscribe_youtube_pubsub;
mod update_bilibili_channel_view_and_subscriber;
mod update_currency_exchange_rate;
mod update_upcoming_stream;
mod update_youtube_channel_donation;
mod update_youtube_channel_view_and_subscriber;
mod upsert_youtube_stream;

use chrono::{DateTime, Utc};
use tokio::sync::mpsc::Sender;
use tracing::{
    field::{debug, display, Empty},
    Instrument, Span,
};
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
        UpdateYoutubeChannelViewAndSubscriber => "update_youtube_channel_view_and_subscriber",
        UpdateBilibiliChannelViewAndSubscriber => "update_bilibili_channel_view_and_subscriber",
        UpdateYoutubeChannelDonation => "update_youtube_channel_donation",
        UpdateCurrencyExchangeRate => "update_currency_exchange_rate",
        UpsertYoutubeStream { .. } => "upsert_youtube_stream",
        CollectYoutubeStreamMetadata { .. } => "collect_youtube_stream_metadata",
        CollectYoutubeStreamLiveChat { .. } => "collect_youtube_stream_live_chat",
        UpdateUpcomingStream => "update_upcoming_stream",
    };

    let span = match &payload {
        HealthCheck => tracing::info_span!("Ignored"),
        _ => tracing::info_span!(
            "Job",
            "span.kind" = "client",
            "name" = Empty,
            //// otel
            "otel.status_code" = "OK",
            //// job
            "job.type" = job_type,
            "job.payload" = Empty,
            //// error
            "error.message" = Empty,
            "error.cause_chain" = Empty,
        ),
    };

    async move {
        let result = match payload {
            HealthCheck => health_check::execute().await,
            RefreshYoutubeRss => refresh_youtube_rss::execute(&pool, hub).await,
            SubscribeYoutubePubsub => subscribe_youtube_pubsub::execute(&pool, hub).await,
            UpdateYoutubeChannelViewAndSubscriber => {
                update_youtube_channel_view_and_subscriber::execute(&pool, hub).await
            }
            UpdateBilibiliChannelViewAndSubscriber => {
                update_bilibili_channel_view_and_subscriber::execute(&pool, hub).await
            }
            // TODO:
            UpdateYoutubeChannelDonation => update_youtube_channel_donation::execute().await,
            // TODO:
            UpdateCurrencyExchangeRate => update_currency_exchange_rate::execute(&pool).await,
            UpsertYoutubeStream(payload) => {
                upsert_youtube_stream::execute(&pool, hub, payload).await
            }
            CollectYoutubeStreamMetadata(payload) => {
                collect_youtube_stream_metadata::execute(&pool, hub, continuation, payload).await
            }
            CollectYoutubeStreamLiveChat(payload) => {
                collect_youtube_stream_live_chat::execute(&pool, hub, continuation, payload).await
            }
            UpdateUpcomingStream => update_upcoming_stream::execute(&pool).await,
        };

        let query = match result {
            Ok(JobResult::Next { run, continuation }) => UpdateJobQuery {
                job_id,
                status: JobStatus::Queued,
                next_run: Some(run),
                continuation,
            },
            Ok(JobResult::Completed) => UpdateJobQuery {
                job_id,
                status: JobStatus::Success,
                next_run: None,
                continuation: None,
            },
            Err(ref err) => {
                eprintln!("[Job Error] {job_type}-{job_id} {err}");

                Span::current()
                    .record("otel.status_code", "ERROR")
                    .record("error.message", display(err))
                    .record("error.cause_chain", debug(err));

                UpdateJobQuery {
                    job_id,
                    status: JobStatus::Failed,
                    next_run: None,
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
