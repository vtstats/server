use std::env;

use anyhow::Result;
use reqwest::{header::AUTHORIZATION, Client};
use serde::Deserialize;
use vtstats_utils::send_request;

use crate::subscription::{Condition, CreatedSubscription, Subscription, Transport};

#[derive(Deserialize)]
struct ListSubscriptionResponse {
    data: Vec<CreatedSubscription>,
}

pub async fn list_subscriptions(token: &str, client: &Client) -> Result<Vec<CreatedSubscription>> {
    let req = client
        .get("https://api.twitch.tv/helix/eventsub/subscriptions")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header("Client-Id", env::var("TWITCH_CLIENT_ID")?);

    let res = send_request!(req)?;

    let list: ListSubscriptionResponse = res.json().await?;

    Ok(list.data)
}

pub async fn create_channel_update_subscription(
    broadcaster_user_id: String,
    token: &str,
    client: &Client,
) -> Result<String> {
    create_subscription(
        Subscription::ChannelUpdate {
            version: "2".into(),
            transport: transport()?,
            condition: Condition {
                broadcaster_user_id,
            },
        },
        token,
        client,
    )
    .await
}

pub async fn create_stream_offline_subscription(
    broadcaster_user_id: String,
    token: &str,
    client: &Client,
) -> Result<String> {
    create_subscription(
        Subscription::StreamOffline {
            version: "1".into(),
            transport: transport()?,
            condition: Condition {
                broadcaster_user_id,
            },
        },
        token,
        client,
    )
    .await
}

pub async fn create_stream_online_subscription(
    broadcaster_user_id: String,
    token: &str,
    client: &Client,
) -> Result<String> {
    create_subscription(
        Subscription::StreamOnline {
            version: "1".into(),
            transport: transport()?,
            condition: Condition {
                broadcaster_user_id,
            },
        },
        token,
        client,
    )
    .await
}

fn transport() -> anyhow::Result<Transport> {
    Ok(Transport {
        method: "webhook".into(),
        callback: format!("https://{}/api/twitch", env::var("SERVER_HOSTNAME")?),
        secret: Some(env::var("TWITCH_WEBHOOK_SECRET")?),
    })
}

async fn create_subscription(
    subscription: Subscription,
    token: &str,
    client: &Client,
) -> Result<String> {
    let req = client
        .post("https://api.twitch.tv/helix/eventsub/subscriptions")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header("Client-Id", env::var("TWITCH_CLIENT_ID")?)
        .json(&subscription);

    let res = send_request!(req)?;

    let mut list: ListSubscriptionResponse = res.json().await?;

    if let Some(s) = list.data.pop() {
        Ok(s.id)
    } else {
        anyhow::bail!("twitch responses nothing");
    }
}
