use std::env;

use reqwest::{header::CONTENT_TYPE, Client};
use serde::Deserialize;
use vtstats_utils::send_request;

#[derive(Deserialize)]
pub struct AccessToken {
    pub access_token: String,
}

pub async fn get_access_token(client: &Client) -> anyhow::Result<AccessToken> {
    let req = client
        .post("https://id.twitch.tv/oauth2/token")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(format!(
            "client_id={}&client_secret={}&grant_type=client_credentials",
            env::var("TWITCH_CLIENT_ID")?,
            env::var("TWITCH_CLIENT_SECRET")?,
        ));

    let res = send_request!(req)?;

    let token = res.json().await?;

    Ok(token)
}
