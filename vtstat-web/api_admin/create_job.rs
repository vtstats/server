use chrono::Utc;
use serde::Deserialize;
use vtstat_database::{
    jobs::{JobPayload, PushJobQuery},
    PgPool,
};
use warp::{reply::Response, Rejection, Reply};

use crate::{api_admin::ActionResponse, reject::WarpError};

#[derive(Deserialize)]
#[serde(rename = "UPPER_CASE")]
#[serde(tag = "kind")]
pub enum CreateJobPayload {
    HealthCheck,
    RefreshYoutubeRss,
    SubscribeYoutubePubsub,
    UpdateYoutubeChannelViewAndSubscriber,
    UpdateBilibiliChannelViewAndSubscriber,
    UpdateYoutubeChannelDonation,
    UpdateCurrencyExchangeRate,
    InstallDiscordCommands,
}

pub async fn create_job(pool: PgPool, payload: CreateJobPayload) -> Result<Response, Rejection> {
    let job_id = PushJobQuery {
        continuation: None,
        next_run: Some(Utc::now()),
        payload: match payload {
            CreateJobPayload::HealthCheck => JobPayload::HealthCheck,
            CreateJobPayload::RefreshYoutubeRss => JobPayload::RefreshYoutubeRss,
            CreateJobPayload::SubscribeYoutubePubsub => JobPayload::SubscribeYoutubePubsub,
            CreateJobPayload::UpdateYoutubeChannelViewAndSubscriber => {
                JobPayload::UpdateYoutubeChannelViewAndSubscriber
            }
            CreateJobPayload::UpdateBilibiliChannelViewAndSubscriber => {
                JobPayload::UpdateBilibiliChannelViewAndSubscriber
            }
            CreateJobPayload::UpdateYoutubeChannelDonation => {
                JobPayload::UpdateYoutubeChannelDonation
            }
            CreateJobPayload::UpdateCurrencyExchangeRate => JobPayload::UpdateCurrencyExchangeRate,
            CreateJobPayload::InstallDiscordCommands => JobPayload::InstallDiscordCommands,
        },
    }
    .execute(&pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&ActionResponse {
        msg: format!("Job#{job_id} created."),
    })
    .into_response())
}
