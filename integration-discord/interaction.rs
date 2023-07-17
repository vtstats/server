use std::collections::HashMap;

use serde::{de, Deserialize, Serialize};
use serde_json::value::RawValue;

/// Discord interaction object
///
/// https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object
#[derive(Debug, PartialEq)]
pub enum Interaction {
    Ping,

    /// https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-structure
    ApplicationCommand {
        guild_id: String,
        channel_id: String,
        data: ApplicationCommandData,
        app_permissions: Permissions,
    },
}

#[derive(Debug, PartialEq)]
pub struct Permissions(u64);

/// https://discord.com/developers/docs/topics/permissions#permissions-bitwise-permission-flags
impl Permissions {
    pub fn can_send_message(&self) -> bool {
        (self.0 & (1 << 11)) != 0
    }

    pub fn can_view_channel(&self) -> bool {
        (self.0 & (1 << 10)) != 0
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ApplicationCommandData {
    pub name: String,
    #[serde(default)]
    pub options: Vec<CommandOption>,
}

impl ApplicationCommandData {
    pub fn option_string(&self, name: &str) -> Option<String> {
        self.options.iter().find_map(|option| match option {
            CommandOption::String { name: n, value } if name == n => Some(value.clone()),
            _ => None,
        })
    }

    pub fn option_integer(&self, name: &str) -> Option<i32> {
        self.options.iter().find_map(|option| match option {
            CommandOption::Integer { name: n, value } if name == n => Some(*value),
            _ => None,
        })
    }
}

/// https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure
#[derive(Debug, PartialEq)]
pub enum CommandOption {
    SubCommand {
        name: String,
    },
    SubCommandGroup {
        name: String,
    },
    String {
        name: String,
        value: String,
    },
    /// Any integer between -2^53 and 2^53,
    Integer {
        name: String,
        value: i32,
    },
    Boolean {
        name: String,
    },
    User {
        name: String,
    },
    /// Includes all channel types + categories,
    Channel {
        name: String,
    },
    Role {
        name: String,
    },
    /// Includes users and roles,
    Mentionable {
        name: String,
    },
    /// Any double between -2^53 and 2^53,
    Number {
        name: String,
    },
    /// attachment object,
    Attachment {
        name: String,
    },
}

// discord use number for tagging, however serde don't support it
// so we did a little hack by using the raw value provided by serde_json
impl<'de> de::Deserialize<'de> for Interaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut map: HashMap<&'de str, &'de RawValue> = Deserialize::deserialize(deserializer)?;

        let mut get = |key: &str| -> Result<_, D::Error> {
            map.remove(key)
                .ok_or_else(|| de::Error::custom(format!("field {:?} not found", key)))
                .map(|raw| raw.get())
        };

        match get("type")?.trim() {
            "1" => Ok(Interaction::Ping),
            "2" => Ok(Interaction::ApplicationCommand {
                data: serde_json::from_str(get("data")?).map_err(de::Error::custom)?,
                guild_id: get("guild_id")?.trim_matches('"').into(),
                channel_id: get("channel_id")?.trim_matches('"').into(),
                app_permissions: get("app_permissions")?
                    .trim_matches('"')
                    .parse::<u64>()
                    .map(Permissions)
                    .map_err(de::Error::custom)?,
            }),
            // 3 => MESSAGE_COMPONENT
            // 4 => APPLICATION_COMMAND_AUTOCOMPLETE
            // 5 => MODAL_SUBMIT
            ty => Err(de::Error::custom(format!("unknown message type {ty}"))),
        }
    }
}

impl<'de> de::Deserialize<'de> for CommandOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut map: HashMap<&'de str, &'de RawValue> = Deserialize::deserialize(deserializer)?;

        let mut get = |key: &str| -> Result<_, D::Error> {
            map.remove(key)
                .ok_or_else(|| de::Error::custom(format!("field {:?} not found", key)))
                .map(|raw| raw.get())
        };

        match get("type")?.trim() {
            "3" => Ok(CommandOption::String {
                name: serde_json::from_str(get("name")?).map_err(de::Error::custom)?,
                value: serde_json::from_str(get("value")?).map_err(de::Error::custom)?,
            }),
            "4" => Ok(CommandOption::Integer {
                name: serde_json::from_str(get("name")?).map_err(de::Error::custom)?,
                value: serde_json::from_str(get("value")?).map_err(de::Error::custom)?,
            }),
            ty => Err(de::Error::custom(format!("unknown message type {ty}"))),
        }
    }
}

/// Response to discord interaction
///
/// https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-response-structure
#[derive(Serialize)]
#[serde(untagged)]
pub enum InteractionResponse {
    Pong {
        #[serde(rename = "type")]
        ty: usize,
    },

    ChannelMessage {
        #[serde(rename = "type")]
        ty: usize,
        data: InteractionCallbackData,
    },
}

/// https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-messages
#[derive(Serialize)]
pub struct InteractionCallbackData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    pub content: String,
}

impl InteractionResponse {
    pub const fn pong() -> Self {
        InteractionResponse::Pong { ty: 1 }
    }

    pub const fn channel_message(data: InteractionCallbackData) -> Self {
        InteractionResponse::ChannelMessage { ty: 4, data }
    }
}

#[test]
fn test() {
    use serde_json::{from_str, to_string};

    // request
    assert_eq!(
        from_str::<Interaction>(r#"{"type":1}"#).unwrap(),
        Interaction::Ping {}
    );

    assert_eq!(
        from_str::<Interaction>(include_str!("./testdata/interaction.0.json")).unwrap(),
        Interaction::ApplicationCommand {
            channel_id: "channel_id".into(),
            guild_id: "guild_id".into(),
            app_permissions: Permissions(0),
            data: ApplicationCommandData {
                name: "command".into(),
                options: vec![CommandOption::String {
                    name: "arg1".into(),
                    value: "test".into()
                }]
            }
        }
    );

    assert_eq!(
        from_str::<Interaction>(include_str!("./testdata/interaction.1.json")).unwrap(),
        Interaction::ApplicationCommand {
            channel_id: "channel_id".into(),
            guild_id: "guild_id".into(),
            app_permissions: Permissions(0),
            data: ApplicationCommandData {
                name: "list".into(),
                options: vec![]
            }
        }
    );

    assert_eq!(
        from_str::<Interaction>(include_str!("./testdata/interaction.2.json")).unwrap(),
        Interaction::ApplicationCommand {
            channel_id: "channel_id".into(),
            guild_id: "guild_id".into(),
            app_permissions: Permissions(0),
            data: ApplicationCommandData {
                name: "remove".into(),
                options: vec![CommandOption::Integer {
                    name: "subscription_id".into(),
                    value: 3
                }]
            }
        }
    );

    // response
    assert_eq!(
        to_string(&InteractionResponse::pong()).unwrap(),
        r#"{"type":1}"#
    );

    assert_eq!(
        to_string(&InteractionResponse::channel_message(
            InteractionCallbackData {
                tts: None,
                content: "Congrats on sending your command!".into()
            }
        ))
        .unwrap(),
        r#"{"type":4,"data":{"content":"Congrats on sending your command!"}}"#
    );

    // permission
    assert!(Permissions(137419730439745).can_send_message());
    assert!(Permissions(137419730439745).can_view_channel());

    assert!(Permissions(137419730438721).can_send_message());
    assert!(!Permissions(137419730438721).can_view_channel());

    assert!(!Permissions(137419730437697).can_send_message());
    assert!(Permissions(137419730437697).can_view_channel());
}
