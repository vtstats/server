use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use vtstats_utils::send_request;

use reqwest::{Client, ClientBuilder, Result};

pub struct DiscordApiCache {
    client: Client,
    guilds: HashMap<String, Guild>,
    guild_roles: HashMap<String, ExpireAt<Vec<Role>>>,
}

pub struct ExpireAt<T> {
    time: DateTime<Utc>,
    item: T,
}

#[derive(Deserialize)]
pub struct Role {
    /// snowflake, role id
    pub id: String,
    /// role name
    pub name: String,
}

impl Role {
    pub fn is_mod(&self) -> bool {
        self.name.eq_ignore_ascii_case("mod") || self.name.eq_ignore_ascii_case("moderator")
    }

    pub fn is_admin(&self) -> bool {
        self.name.eq_ignore_ascii_case("admin") || self.name.eq_ignore_ascii_case("administrator")
    }
}

#[derive(Deserialize)]
pub struct Guild {
    /// Owner id
    pub owner_id: String,
}

impl DiscordApiCache {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(DiscordApiCache {
            client: ClientBuilder::new()
                .http1_only()
                .brotli(true)
                .deflate(true)
                .gzip(true)
                .build()
                .expect("create http client"),
            guild_roles: HashMap::with_capacity(10),
            guilds: HashMap::with_capacity(10),
        }))
    }

    pub async fn get_guild<'a>(&'a mut self, guild_id: &str) -> Result<Option<&'a Guild>> {
        if !self.guilds.contains_key(guild_id) {
            let url = format!("https://discord.com/api/v10/guilds/{guild_id}");

            let req = self.client.get(url).header(
                reqwest::header::AUTHORIZATION,
                format!("Bot {}", std::env::var("DISCORD_BOT_TOKEN").unwrap()),
            );

            let res = send_request!(req)?;

            let guild: Guild = res.json().await?;

            self.guilds.insert(guild_id.into(), guild);
        }

        Ok(self.guilds.get(guild_id))
    }

    pub async fn get_guild_role<'a>(&'a mut self, guild_id: &str) -> Result<Option<&'a Vec<Role>>> {
        let now = Utc::now();

        let existed = match self.guild_roles.get(guild_id) {
            Some(i) => i.time < now,
            None => false,
        };

        if !existed {
            let url = format!("https://discord.com/api/v10/guilds/{guild_id}/roles");

            let req = self.client.get(url).header(
                reqwest::header::AUTHORIZATION,
                format!("Bot {}", std::env::var("DISCORD_BOT_TOKEN").unwrap()),
            );

            let res = send_request!(req)?;

            let roles: Vec<Role> = res.json().await?;

            self.guild_roles.insert(
                guild_id.into(),
                ExpireAt {
                    time: now + Duration::minutes(15),
                    item: roles,
                },
            );
        }

        Ok(self.guild_roles.get(guild_id).map(|x| &x.item))
    }
}
