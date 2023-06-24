use vtstat_database::subscriptions::{
    CreateDiscordSubscriptionQuery, DiscordSubscriptionPayload, RemoveDiscordSubscriptionQuery,
};
use vtstat_database::{subscriptions::ListDiscordSubscriptionQuery, PgPool};
use warp::Reply;
use warp::{reply::Response, Filter, Rejection};

use integration_discord::interaction::{Interaction, InteractionCallbackData, InteractionResponse};
use integration_discord::verify::verify_request;

use crate::filters::with_pool;
use crate::reject::WarpError;

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("discord")
        .and(warp::post())
        .and(with_pool(pool))
        .and(verify_request())
        .and_then(discord_interactions)
}

pub async fn discord_interactions(
    pool: PgPool,
    update: Interaction,
) -> Result<Response, Rejection> {
    match update {
        Interaction::Ping => Ok(warp::reply::json(&InteractionResponse::pong()).into_response()),
        Interaction::ApplicationCommand { channel_id, data } => match data.name.as_str() {
            "list" => list_subscriptions(&pool, channel_id).await,
            "add" => {
                create_subscription(
                    &pool,
                    channel_id,
                    data.option_string("vtuber_id")
                        .ok_or_else(|| warp::reject())?,
                )
                .await
            }
            "remove" => {
                remove_subscription(
                    &pool,
                    channel_id,
                    data.option_integer("subscription_id")
                        .ok_or_else(|| warp::reject())?,
                )
                .await
            }
            _ => Err(warp::reject()),
        },
    }
}

pub async fn list_subscriptions(pool: &PgPool, channel_id: String) -> Result<Response, Rejection> {
    let subscriptions = ListDiscordSubscriptionQuery::ByChannelId(channel_id)
        .execute(&pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData {
            tts: None,
            content: format!("{:?}", subscriptions),
        },
    ))
    .into_response())
}

pub async fn create_subscription(
    pool: &PgPool,
    channel_id: String,
    vtuber_id: String,
) -> Result<Response, Rejection> {
    let id = CreateDiscordSubscriptionQuery {
        payload: DiscordSubscriptionPayload {
            channel_id,
            vtuber_id,
        },
    }
    .execute(pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData {
            tts: None,
            content: format!("Subscription created w/ id {id}"),
        },
    ))
    .into_response())
}

pub async fn remove_subscription(
    pool: &PgPool,
    channel_id: String,
    subscription_id: i32,
) -> Result<Response, Rejection> {
    let _ = RemoveDiscordSubscriptionQuery {
        channel_id,
        subscription_id,
    }
    .execute(pool)
    .await
    .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData {
            tts: None,
            content: format!("Subscription removed"),
        },
    ))
    .into_response())
}
