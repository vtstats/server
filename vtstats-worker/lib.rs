#![deny(clippy::unwrap_used)]

use chrono::{DateTime, Duration, TimeZone, Utc};
use reqwest::{ClientBuilder, Proxy};
use std::env;
use tokio::{
    sync::mpsc::{channel, Sender},
    sync::oneshot::Receiver,
    time::sleep,
};
use vtstats_database::{
    jobs::{next_queued, pull_jobs},
    PgListener, PgPool,
};

pub mod jobs;

pub async fn main(shutdown_rx: Receiver<()>) -> anyhow::Result<()> {
    let (shutdown_complete_tx, mut shutdown_complete_rx) = channel(1);

    tokio::select! {
        res = execute(shutdown_complete_tx) => {
            if let Err(err) = res {
                eprintln!("[Polling Error] {err:?}");
            }
        },
        _ = async { shutdown_rx.await.ok() } => {},
    };

    tracing::warn!("Shutting down worker...");

    // wait for all spawned tasks to complete
    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}

async fn execute(shutdown_complete_tx: Sender<()>) -> anyhow::Result<()> {
    let database_url = &env::var("DATABASE_URL")?;

    let pool = PgPool::connect(database_url).await?;

    let mut listener = PgListener::connect(database_url).await?;

    listener.listen("vt_new_job_queued").await?;

    let mut builder = ClientBuilder::new()
        .http1_only()
        .brotli(true)
        .deflate(true)
        .gzip(true);

    if let Ok(proxy) = env::var("ALL_PROXY") {
        builder = builder.proxy(Proxy::all(proxy).unwrap());
    }

    let client = builder.build()?;

    tracing::warn!("Start executing jobs...");

    loop {
        for job in pull_jobs(&pool).await? {
            tokio::spawn(jobs::execute(
                job,
                pool.clone(),
                client.clone(),
                shutdown_complete_tx.clone(),
            ));
        }

        waiting(&pool, &mut listener).await?;
    }
}

async fn waiting(pool: &PgPool, listener: &mut PgListener) -> anyhow::Result<()> {
    let mut next_queued_at = next_queued(pool)
        .await?
        .unwrap_or_else(|| Utc::now() + Duration::minutes(1));

    loop {
        let now = Utc::now();

        if next_queued_at <= now {
            return Ok(());
        }

        let timeout = (next_queued_at - now).to_std()?;
        tokio::select! {
            _ = sleep(timeout) => return Ok(()),

            notification = listener.try_recv() => {
                if let Some(queued) = notification?.and_then(|n| parse_timestamp(n.payload())) {
                    next_queued_at = std::cmp::min(queued, next_queued_at);
                } else {
                    next_queued_at = now + Duration::seconds(1);
                }
            }
        }
    }
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(value.parse().ok()?).single()
}
