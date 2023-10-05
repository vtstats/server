use chrono::{Duration, DurationRound, Utc};
use reqwest::Client;
use serde::Deserialize;
use vtstats_database::{exchange_rates::update_exchange_rates, PgPool};

use super::JobResult;

pub async fn execute(pool: &PgPool, client: Client) -> anyhow::Result<JobResult> {
    let now = Utc::now().duration_trunc(Duration::hours(1))?;

    #[derive(Deserialize)]
    struct Rate {
        #[serde(rename = "isoA3Code")]
        code: String,
        value: f32,
    }

    let res = client
        .get("https://ec.europa.eu/budg/inforeuro/api/public/monthly-rates")
        .send()
        .await?;

    let rates: Vec<Rate> = res.json().await?;

    update_exchange_rates(pool, now, rates.into_iter().map(|r| (r.code, r.value))).await?;

    Ok(JobResult::Next {
        run: now + Duration::days(10),
    })
}
