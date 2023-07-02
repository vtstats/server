use std::env;
use std::time::Duration;
use tokio::{
    sync::mpsc::{channel, Sender},
    time::sleep,
};
use vtstat_database::{jobs::PullJobQuery, PgPool};
use vtstat_request::RequestHub;

mod jobs;

pub async fn main() -> anyhow::Result<()> {
    let (shutdown_complete_tx, mut shutdown_complete_rx) = channel(1);

    tokio::select! {
        res = polling(shutdown_complete_tx) => {
            if let Err(err) = res {
                eprintln!("[Polling Error] {err:?}");
            }
        },
        _ = vtstat_utils::shutdown::signal() => {},
    };

    println!("Shuting down...");

    // wait for all spawned tasks to complete
    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}

async fn polling(shutdown_complete_tx: Sender<()>) -> anyhow::Result<()> {
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let hub = RequestHub::new();

    println!("Start polling jobs...");

    loop {
        let jobs = PullJobQuery { limit: 5 }.execute(&pool).await?;
        let reached_limit = jobs.len() == 5;

        for job in jobs.into_iter() {
            tokio::spawn(jobs::execute(
                job,
                pool.clone(),
                hub.clone(),
                shutdown_complete_tx.clone(),
            ));
        }

        // pull new jobs immediately if reached limit
        if !reached_limit {
            sleep(Duration::from_millis(500)).await // 500ms
        }
    }
}
