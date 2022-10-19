use reqwest::{ClientBuilder, StatusCode};
use std::{env, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vtstat_utils::dotenv::load();

    let address = &env::var("SERVER_ADDRESS")?;

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(2))
        .build()?;

    let res = client
        .get(format!("http://{address}/api/whoami"))
        .send()
        .await?;

    let status = res.status();

    if status != StatusCode::OK {
        anyhow::bail!(
            "Health check failed: received {} {}",
            status.as_u16(),
            status.as_str()
        );
    }

    Ok(())
}
