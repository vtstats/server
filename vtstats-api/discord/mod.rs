use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use integration_discord::interaction::{
    ApplicationCommandData, Interaction, InteractionCallbackData, InteractionResponse, Member,
    Permissions,
};
use integration_discord::verify;
use integration_discord::DiscordApiCache;
use vtstats_database::subscriptions::{
    CreateDiscordSubscriptionQuery, DiscordSubscriptionPayload, ListDiscordSubscriptionQuery,
    RemoveDiscordSubscriptionQuery,
};
use vtstats_database::PgPool;

#[derive(Clone)]
struct DiscordRouteState {
    pool: PgPool,
    cache: Arc<Mutex<DiscordApiCache>>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", post(discord_interactions))
        .layer(
            ServiceBuilder::new()
                .map_request_body(axum::body::boxed)
                .layer(axum::middleware::from_fn(verify)),
        )
        .with_state(DiscordRouteState {
            pool,
            cache: DiscordApiCache::new(),
        })
}

async fn discord_interactions(
    State(DiscordRouteState { pool, cache }): State<DiscordRouteState>,
    Json(update): Json<Interaction>,
) -> impl IntoResponse {
    match update {
        Interaction::Ping => Json(InteractionResponse::pong()),
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

            Json(InteractionResponse::channel_message(
                InteractionCallbackData { tts: None, content },
            ))
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
        "list" => list_subscriptions(pool, guild_id, channel_id).await,
        "list_all" => list_all_subscriptions(pool, guild_id).await,
        "add" | "remove" => {
            check_permission(&guild_id, &app_permissions, &member, cache).await?;

            let vtuber_id = data
                .option_string("vtuber_id")
                .ok_or_else(|| anyhow::anyhow!("Can't get option `vtuber_id`"))?;

            if command == "add" {
                create_subscription(pool, guild_id, channel_id, vtuber_id).await
            } else {
                remove_subscription(pool, guild_id, channel_id, vtuber_id).await
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
        anyhow::bail!("can't get guild info of `{guild_id}`");
    };

    if guild.owner_id != member.user.id {
        let roles = cache.get_guild_role(guild_id).await?;

        let Some(roles) = roles else {
            anyhow::bail!("can't get roles info of `{guild_id}`");
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
        return Ok("no subscription found in this server, use /add to create one.".to_string());
    }

    subscriptions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    let mut s = format!(
        "{} subscription(s) found in this server:\n",
        subscriptions.len(),
    );
    for sub in subscriptions {
        s += &format!(
            "- vtuber: `{}` channel: <#{}> created: <t:{}>\n",
            sub.payload.vtuber_id,
            sub.payload.channel_id,
            sub.created_at.timestamp()
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
        return Ok("no subscription found in this channel, use /add to create one.".to_string());
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
