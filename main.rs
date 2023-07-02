#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = std::env::args().nth(1).unwrap_or_default();

    vtstat_utils::dotenv::load();
    vtstat_utils::tracing::init();

    match command.as_str() {
        "standalone" => tokio::try_join!(vtstat_web::main(), vtstat_worker::main()).map(|_| ()),
        "web" => vtstat_web::main().await,
        "worker" => vtstat_worker::main().await,
        "database-migrate" => vtstat_database::migrate().await,
        c => anyhow::bail!("Unknown command {:?}", c),
    }
}
