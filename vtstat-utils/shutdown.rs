use tokio::signal::unix;

pub async fn signal() {
    let (Ok(mut sigint), Ok(mut sigterm)) = (
        unix::signal(unix::SignalKind::interrupt()),
        unix::signal(unix::SignalKind::terminate()),
    )  else {
        eprintln!("Failed to listen unix signal");
        return;
    };

    tokio::select! {
        _ = sigint.recv() => {
            eprintln!("Received SIGINT signal, shutting down...");
        },
        _ = sigterm.recv() => {
            eprintln!("Received SIGTERM signal, shutting down...");
        },
    };
}
