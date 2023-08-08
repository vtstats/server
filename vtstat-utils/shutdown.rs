use tokio::signal::unix;
use tokio::sync::oneshot::Sender;

pub async fn shutdown(tx_iter: impl IntoIterator<Item = Sender<()>>) -> anyhow::Result<()> {
    let (Ok(mut sigint), Ok(mut sigterm)) = (
        unix::signal(unix::SignalKind::interrupt()),
        unix::signal(unix::SignalKind::terminate()),
    )  else {
        anyhow::bail!("Failed to listen unix signal")
    };

    tokio::select! {
        _ = sigint.recv() => {
            tracing::warn!("Received SIGINT signal...");
        },
        _ = sigterm.recv() => {
            tracing::warn!("Received SIGTERM signal...");
        },
    };

    for tx in tx_iter {
        let _ = tx.send(());
    }

    Ok(())
}
