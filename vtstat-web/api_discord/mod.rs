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
        Interaction::ApplicationCommand {
            channel_id,
            data,
            guild_id,
        } => match data.name.as_str() {
            "list" => Ok(list_subscriptions(&pool, guild_id, channel_id).await),
            "list_all" => Ok(list_all_subscriptions(&pool, guild_id).await),
            "add" => Ok(create_subscription(
                &pool,
                guild_id,
                channel_id,
                data.option_string("vtuber_id")
                    .ok_or_else(|| warp::reject())?,
            )
            .await),
            "remove" => Ok(remove_subscription(
                &pool,
                guild_id,
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

pub async fn list_all_subscriptions(pool: &PgPool, guild_id: String) -> Response {
    let result = ListDiscordSubscriptionQuery::ByGuildId { guild_id }
        .execute(&pool)
        .await;

    let content = match result {
        Ok(subscriptions) if subscriptions.is_empty() => {
            format!("No subscriptions found in this server, use /add to create one.")
        }
        Ok(mut subscriptions) => {
            subscriptions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            let mut s = format!(
                "{} subscription(s) found in this server:\n",
                subscriptions.len(),
            );
            for sub in subscriptions {
                s += &format!(
                    "- *id*: `{id}` *vtuber:* `{vtuber}` *channel:* <#{channel}> *created:* <t:{ts}>\n",
                    id = sub.subscription_id,
                    vtuber = sub.payload.vtuber_id,
                    channel = sub.payload.channel_id,
                    ts = sub.created_at.timestamp()
                );
            }
            s
        }
        Err(err) => format!("Error: {}", err),
    };

    warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData { tts: None, content },
    ))
    .into_response()
}

pub async fn list_subscriptions(pool: &PgPool, guild_id: String, channel_id: String) -> Response {
    let result = ListDiscordSubscriptionQuery::ByChannelId {
        channel_id,
        guild_id,
    }
    .execute(&pool)
    .await;

    let content = match result {
        Ok(subscriptions) if subscriptions.is_empty() => {
            format!("No subscriptions found in this channel, use /add to create one.")
        }
        Ok(mut subscriptions) => {
            subscriptions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            let mut s = format!(
                "{} subscription(s) found in <#{}>:\n",
                subscriptions.len(),
                subscriptions[0].payload.channel_id
            );
            for sub in subscriptions {
                s += &format!(
                    "- *id*: `{id}` *vtuber:* `{vtuber}` *created:* <t:{ts}>\n",
                    id = sub.subscription_id,
                    vtuber = sub.payload.vtuber_id,
                    ts = sub.created_at.timestamp()
                );
            }
            s
        }
        Err(err) => format!("Error: {}", err),
    };

    warp::reply::json(&InteractionResponse::channel_message(
        InteractionCallbackData { tts: None, content },
    ))
    .into_response()
}

pub async fn create_subscription(
    pool: &PgPool,
    guild_id: String,
    channel_id: String,
    vtuber_id: String,
) -> Response {
    let result = CreateDiscordSubscriptionQuery {
        payload: DiscordSubscriptionPayload {
            guild_id,
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
    guild_id: String,
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
