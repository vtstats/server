use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Reply;
use warp::{reply::Response, Filter, Rejection};

use integration_discord::interaction::{
    ApplicationCommandData, Interaction, InteractionCallbackData, InteractionResponse, Member,
    Permissions,
};
use integration_discord::{validate, DiscordApiCache};
use vtstat_database::subscriptions::{
    CreateDiscordSubscriptionQuery, DiscordSubscriptionPayload, ListDiscordSubscriptionQuery,
    RemoveDiscordSubscriptionQuery,
};
use vtstat_database::PgPool;

pub fn routes(
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cache = DiscordApiCache::new();

    warp::path!("discord")
        .and(warp::post())
        .and(validate())
        .and_then(move |update| discord_interactions(pool.clone(), update, cache.clone()))
}

async fn discord_interactions(
    pool: PgPool,
    update: Interaction,
    cache: Arc<Mutex<DiscordApiCache>>,
) -> Result<Response, Rejection> {
    match update {
        Interaction::Ping => Ok(warp::reply::json(&InteractionResponse::pong()).into_response()),
        Interaction::ApplicationCommand {
            channel_id,
            data,
            guild_id,
            app_permissions,
            member,
        } => {
            let content = handle_command(
                guild_id,
                channel_id,
                &data,
                app_permissions,
                member,
                &pool,
                cache,
            )
            .await
            .unwrap_or_else(|err| format!("Error: {err}"));

            Ok(warp::reply::json(&InteractionResponse::channel_message(
                InteractionCallbackData { tts: None, content },
            ))
            .into_response())
        }
    }
}

async fn handle_command(
    guild_id: String,
    channel_id: String,
    data: &ApplicationCommandData,
    app_permissions: Permissions,
    member: Member,
    pool: &PgPool,
    cache: Arc<Mutex<DiscordApiCache>>,
) -> anyhow::Result<String> {
    let command = data.name.as_str();

    match command {
        "list" => list_subscriptions(&pool, guild_id, channel_id).await,
        "list_all" => list_all_subscriptions(&pool, guild_id).await,
        "add" | "remove" => {
            check_permission(&guild_id, &app_permissions, &member, cache).await?;

            let vtuber_id = data
                .option_string("vtuber_id")
                .ok_or_else(|| anyhow::anyhow!("Can't get option `vtuber_id`"))?;

            if command == "add" {
                create_subscription(&pool, guild_id, channel_id, vtuber_id).await
            } else {
                remove_subscription(&pool, guild_id, channel_id, vtuber_id).await
            }
        }
        _ => Ok(format!("Error: unknown command {command:?}")),
    }
}

async fn check_permission(
    guild_id: &str,
    app_permissions: &Permissions,
    member: &Member,
    cache: Arc<Mutex<DiscordApiCache>>,
) -> anyhow::Result<()> {
    let mut cache = cache.lock().await;

    let guild = cache.get_guild(guild_id).await?;

    let Some(guild) = guild else {
        anyhow::bail!("Can't get guild info of `{guild_id}`");
    };

    if guild.owner_id != member.user.id {
        let roles = cache.get_guild_role(guild_id).await?;

        let Some(roles) = roles else {
            anyhow::bail!("Can't get roles info of `{guild_id}`");
        };

        if !roles
            .iter()
            .filter(|role| member.roles.contains(&role.id))
            .any(|role| role.is_admin() || role.is_mod())
        {
            anyhow::bail!(
                "to use this command, you need to be server owner, \
                or have role `Admin[istrator]` or `Mod[erator]`."
            );
        }
    }

    if !app_permissions.can_view_channel() {
        anyhow::bail!(
            "sorry, I don't have permission to **send messages** \
            in this channel. Please check your settings."
        );
    }

    if !app_permissions.can_view_channel() {
        anyhow::bail!(
            "sorry, I don't have permission to **view \
            this channel**. Please check your settings."
        );
    }

    Ok(())
}

async fn list_all_subscriptions(pool: &PgPool, guild_id: String) -> anyhow::Result<String> {
    let mut subscriptions = ListDiscordSubscriptionQuery::ByGuildId { guild_id }
        .execute(pool)
        .await?;

    if subscriptions.is_empty() {
        return Ok("No subscription found in this server, use /add to create one.".to_string());
    }

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

    Ok(s)
}

async fn list_subscriptions(
    pool: &PgPool,
    guild_id: String,
    channel_id: String,
) -> anyhow::Result<String> {
    let mut subscriptions = ListDiscordSubscriptionQuery::ByChannelId {
        channel_id,
        guild_id,
    }
    .execute(pool)
    .await?;

    if subscriptions.is_empty() {
        return Ok("No subscription found in this channel, use /add to create one.".to_string());
    }

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

    Ok(s)
}

async fn create_subscription(
    pool: &PgPool,
    guild_id: String,
    channel_id: String,
    vtuber_id: String,
) -> anyhow::Result<String> {
    CreateDiscordSubscriptionQuery {
        payload: DiscordSubscriptionPayload {
            guild_id,
            channel_id,
            vtuber_id: vtuber_id.clone(),
        },
    }
    .execute(pool)
    .await?;

    Ok(format!("Success: subscription `{vtuber_id}` created."))
}

async fn remove_subscription(
    pool: &PgPool,
    guild_id: String,
    channel_id: String,
    vtuber_id: String,
) -> anyhow::Result<String> {
    RemoveDiscordSubscriptionQuery {
        guild_id,
        channel_id,
        vtuber_id: vtuber_id.clone(),
    }
    .execute(pool)
    .await?;

    Ok(format!("Success: subscription `{vtuber_id}` removed."))
}
