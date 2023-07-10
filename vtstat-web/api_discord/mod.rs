use vtstat_database::subscriptions::{
    CreateDiscordSubscriptionQuery, DiscordSubscriptionPayload, RemoveDiscordSubscriptionQuery,
};
use vtstat_database::{subscriptions::ListDiscordSubscriptionQuery, PgPool};
use warp::Reply;
use warp::{reply::Response, Filter, Rejection};

use integration_discord::interaction::{Interaction, InteractionCallbackData, InteractionResponse};
use integration_discord::validate;

use crate::filters::with_pool;

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("discord")
        .and(warp::post())
        .and(with_pool(pool))
        .and(validate())
        .and_then(discord_interactions)
}

pub async fn discord_interactions(
    pool: PgPool,
    update: Interaction,
) -> Result<Response, Rejection> {
    match update {
        Interaction::Ping => Ok(warp::reply::json(&InteractionResponse::pong()).into_response()),
        Interaction::ApplicationCommand { channel_id, data } => match data.name.as_str() {
            "list" => Ok(list_subscriptions(&pool, channel_id).await),
            "add" => Ok(create_subscription(
                &pool,
                channel_id,
                data.option_string("vtuber_id")
                    .ok_or_else(|| warp::reject())?,
            )
            .await),
            "remove" => Ok(remove_subscription(
                &pool,
                channel_id,
                data.option_integer("subscription_id")
                    .ok_or_else(|| warp::reject())?,
            )
            .await),
            command => Ok(warp::reply::json(&InteractionResponse::channel_message(
                InteractionCallbackData {
                    tts: None,
                    content: format!("Unknown command {command:?}"),
                },
            ))
            .into_response()),
        },
    }
}

pub async fn list_subscriptions(pool: &PgPool, channel_id: String) -> Response {
    let result = ListDiscordSubscriptionQuery::ByChannelId(channel_id)
        .execute(&pool)
        .await;

    let content = match result {
        Ok(subscriptions) => format!("{:?}", subscriptions),
        Err(err) => format!("Error: {}", err),
    };

    warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData { tts: None, content },
    ))
    .into_response()
}

pub async fn create_subscription(pool: &PgPool, channel_id: String, vtuber_id: String) -> Response {
    let result = CreateDiscordSubscriptionQuery {
        payload: DiscordSubscriptionPayload {
            channel_id,
            vtuber_id,
        },
    }
    .execute(pool)
    .await;

    let content = match result {
        Ok(id) => format!("Success: subscription created w/ id {id}"),
        Err(err) => format!("Error: {}", err),
    };

    warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData { tts: None, content },
    ))
    .into_response()
}

pub async fn remove_subscription(
    pool: &PgPool,
    channel_id: String,
    subscription_id: i32,
) -> Response {
    let result = RemoveDiscordSubscriptionQuery {
        channel_id,
        subscription_id,
    }
    .execute(pool)
    .await;

    let content = match result {
        Ok(_) => format!("Success: subscription removed"),
        Err(err) => format!("Error: {}", err),
    };

    warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData { tts: None, content },
    ))
    .into_response()
}
