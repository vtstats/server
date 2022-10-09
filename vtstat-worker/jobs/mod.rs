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
    Failed {
        reason: String,
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
    };

    let span = tracing::info_span!(
        "job",
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
    );

    let pool_ = pool.clone();

    let job_result = async move {
        match payload {
            HealthCheck => health_check::execute().await,
            RefreshYoutubeRss => refresh_youtube_rss::execute(pool_, hub).await,
            SubscribeYoutubePubsub => subscribe_youtube_pubsub::execute(pool_, hub).await,
            UpdateYoutubeChannelViewAndSubscriber => {
                update_youtube_channel_view_and_subscriber::execute(pool_, hub).await
            }
            UpdateBilibiliChannelViewAndSubscriber => {
                update_bilibili_channel_view_and_subscriber::execute(pool_, hub).await
            }
            // TODO:
            UpdateYoutubeChannelDonation => update_youtube_channel_donation::execute().await,
            // TODO:
            UpdateCurrencyExchangeRate => update_currency_exchange_rate::execute(pool_).await,
            UpsertYoutubeStream(payload) => {
                upsert_youtube_stream::execute(pool_, hub, payload).await
            }
            CollectYoutubeStreamMetadata(payload) => {
                collect_youtube_stream_metadata::execute(pool_, hub, continuation, payload).await
            }
            CollectYoutubeStreamLiveChat(payload) => {
                collect_youtube_stream_live_chat::execute(pool_, hub, continuation, payload).await
            }
        }
        .unwrap_or_else(|ref err| {
            Span::current()
                .record("otel.status_code", "ERROR")
                .record("error.message", display(err))
                .record("error.cause_chain", debug(err));

            JobResult::Failed {
                reason: format!("{err}"),
            }
        })
    }
    .instrument(span)
    .await;

    let db_result = match job_result {
        JobResult::Next { run, continuation } => UpdateJobQuery {
            job_id,
            status: JobStatus::Queued,
            next_run: Some(run),
            continuation,
        },
        JobResult::Completed => UpdateJobQuery {
            job_id,
            status: JobStatus::Success,
            next_run: None,
            continuation: None,
        },
        JobResult::Failed { reason } => {
            eprintln!("[Job Error] {job_type}-{job_id} {reason}");

            UpdateJobQuery {
                job_id,
                status: JobStatus::Failed,
                next_run: None,
                continuation: None,
            }
        }
    }
    .execute(&pool)
    .await;

    if let Err(err) = db_result {
        eprintln!("[Database Error] {err:?}");
    }
}
