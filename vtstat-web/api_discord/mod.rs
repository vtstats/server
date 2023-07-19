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
            app_permissions,
        } => {
            let command = data.name.as_str();
            let content = match command {
                "list" => list_subscriptions(&pool, guild_id, channel_id).await,
                "list_all" => list_all_subscriptions(&pool, guild_id).await,
                "add" | "remove" => {
                    if !app_permissions.can_view_channel() {
                        "Error: sorry, I don't have permission to send messages \
                            in this channel. Please check your settings."
                            .to_string()
                    } else if !app_permissions.can_view_channel() {
                        "Error: sorry, I don't have permission to view \
                            this channel. Please check your settings."
                            .to_string()
                    } else {
                        let vtuber_id = data.option_string("vtuber_id").ok_or_else(warp::reject)?;

                        if command == "add" {
                            create_subscription(&pool, guild_id, channel_id, vtuber_id).await
                        } else {
                            remove_subscription(&pool, guild_id, channel_id, vtuber_id).await
                        }
                    }
                }
                _ => format!("Error: unknown command {command:?}"),
            };

            Ok(warp::reply::json(&InteractionResponse::channel_message(
                InteractionCallbackData { tts: None, content },
            ))
            .into_response())
        }
    }
}

pub async fn list_all_subscriptions(pool: &PgPool, guild_id: String) -> String {
    let result = ListDiscordSubscriptionQuery::ByGuildId { guild_id }
        .execute(pool)
        .await;

    match result {
        Ok(subscriptions) if subscriptions.is_empty() => {
            "No subscription found in this server, use /add to create one.".to_string()
        }
        Ok(mut subscriptions) => {
            subscriptions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            let mut s = format!(
                "{} subscription(s) found in this server:\n",
                subscriptions.len(),
            );
            for sub in subscriptions {
                s += &format!(
                    "- *vtuber:* `{vtuber}` *channel:* <#{channel}> *created:* <t:{ts}>\n",
                    vtuber = sub.payload.vtuber_id,
                    channel = sub.payload.channel_id,
                    ts = sub.created_at.timestamp()
                );
            }
            s
        }
        Err(err) => format!("Error: {}", err),
    }
}

pub async fn list_subscriptions(pool: &PgPool, guild_id: String, channel_id: String) -> String {
    let result = ListDiscordSubscriptionQuery::ByChannelId {
        channel_id,
        guild_id,
    }
    .execute(pool)
    .await;

    match result {
        Ok(subscriptions) if subscriptions.is_empty() => {
            "No subscription found in this channel, use /add to create one.".to_string()
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
                    "- *vtuber:* `{vtuber}` *created:* <t:{ts}>\n",
                    vtuber = sub.payload.vtuber_id,
                    ts = sub.created_at.timestamp()
                );
            }
            s
        }
        Err(err) => format!("Error: {}", err),
    }
}

pub async fn create_subscription(
    pool: &PgPool,
    guild_id: String,
    channel_id: String,
    vtuber_id: String,
) -> String {
    let result = CreateDiscordSubscriptionQuery {
        payload: DiscordSubscriptionPayload {
            guild_id,
            channel_id,
            vtuber_id: vtuber_id.clone(),
        },
    }
    .execute(pool)
    .await;

    match result {
        Ok(_) => format!("Success: subscription `{}` created.", vtuber_id),
        Err(err) => format!("Error: {}", err),
    }
}

pub async fn remove_subscription(
    pool: &PgPool,
    guild_id: String,
    channel_id: String,
    vtuber_id: String,
) -> String {
    let result = RemoveDiscordSubscriptionQuery {
        guild_id,
        channel_id,
        vtuber_id: vtuber_id.clone(),
    }
    .execute(pool)
    .await;

    match result {
        Ok(_) => format!("Success: subscription `{}` removed.", vtuber_id),
        Err(err) => format!("Error: {}", err),
    }
}
