use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = std::env::args().nth(1).unwrap_or_default();

    if command == "database-migrate" {
        vtstats_database::migrate().await?;
        println!("Database migration success");
        return Ok(());
    }

    vtstats_utils::metrics::install();
    vtstats_utils::tracing::init();

    match command.as_str() {
        "standalone" => {
            let (tx1, rx1) = oneshot::channel::<()>();
            let (tx2, rx2) = oneshot::channel::<()>();
            tokio::try_join!(
                vtstats_web::main(rx1),
                vtstats_worker::main(rx2),
                vtstats_utils::shutdown([tx1, tx2])
            )?;
        }
        "web" => {
            let (tx, rx) = oneshot::channel::<()>();
            tokio::try_join!(vtstats_web::main(rx), vtstats_utils::shutdown([tx]))?;
        }
        "worker" => {
            let (tx, rx) = oneshot::channel::<()>();
            tokio::try_join!(vtstats_worker::main(rx), vtstats_utils::shutdown([tx]))?;
        }
        c => anyhow::bail!("Unknown command {:?}", c),
    };

    Ok(())
}
