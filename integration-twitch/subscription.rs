use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatedSubscription {
    pub id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[serde(flatten)]
    pub subscription: Subscription,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Subscription {
    /// https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#streamonline
    #[serde(rename = "stream.online")]
    StreamOnline {
        version: String,
        transport: Transport,
        condition: Condition,
    },
    /// https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#streamoffline
    #[serde(rename = "stream.offline")]
    StreamOffline {
        version: String,
        transport: Transport,
        condition: Condition,
    },
    /// https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelupdate
    #[serde(rename = "channel.update")]
    ChannelUpdate {
        version: String,
        transport: Transport,
        condition: Condition,
    },
}

/// https://dev.twitch.tv/docs/eventsub/eventsub-reference/#stream-online-condition
/// https://dev.twitch.tv/docs/eventsub/eventsub-reference/#stream-offline-condition
/// https://dev.twitch.tv/docs/eventsub/eventsub-reference/#channel-upate-condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub broadcaster_user_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transport {
    pub method: String,
    pub callback: String,
    #[serde(default)]
    pub secret: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelUpdateEvent {
    // The broadcaster’s user ID.
    pub broadcaster_user_id: String,
    // The broadcaster’s user login.
    pub broadcaster_user_login: String,
    // The broadcaster’s user display name.
    pub broadcaster_user_name: String,
    // The channel’s stream title.
    pub title: String,
    // The channel’s broadcast language.
    pub language: String,
    // The channel’s category ID.
    pub category_id: String,
    // The category name.
    pub category_name: String,
    // Array of content classification label IDs currently applied on the Channel. To retrieve a list of all possible IDs, use the Get Content Classification Labels API endpoint.
    pub content_classification_labels: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// https://dev.twitch.tv/docs/eventsub/eventsub-reference/#stream-online-event
pub struct StreamOnlineEvent {
    /// The id of the stream.
    pub id: String,
    /// The broadcaster’s user id.
    pub broadcaster_user_id: String,
    /// The broadcaster’s user login.
    pub broadcaster_user_login: String,
    /// The broadcaster’s user display name.
    pub broadcaster_user_name: String,
    /// The stream type. Valid values are: live, playlist, watch_party, premiere, rerun.
    #[serde(rename = "type")]
    pub type_: String,
    /// The timestamp at which the stream went online at.
    pub started_at: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// https://dev.twitch.tv/docs/eventsub/eventsub-reference/#stream-offline-event
pub struct StreamOfflineEvent {
    /// The broadcaster’s user id.
    pub broadcaster_user_id: String,
    /// The broadcaster’s user login.
    pub broadcaster_user_login: String,
    /// The broadcaster’s user display name.
    pub broadcaster_user_name: String,
}

pub enum Event {
    ChannelUpdateEvent(ChannelUpdateEvent),
    StreamOnlineEvent(StreamOnlineEvent),
    StreamOfflineEvent(StreamOfflineEvent),
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct RawEventSubscription<'a> {
            #[serde(borrow, rename = "type")]
            ty: Cow<'a, str>,
        }

        #[derive(Deserialize)]
        struct RawEvent<'a> {
            #[serde(borrow)]
            event: &'a RawValue,
            subscription: RawEventSubscription<'a>,
        }

        let event = RawEvent::deserialize(deserializer)?;

        match &*event.subscription.ty {
            "stream.online" => serde_json::from_str(event.event.get())
                .map_err(Error::custom)
                .map(Event::StreamOnlineEvent),
            "stream.offline" => serde_json::from_str(event.event.get())
                .map_err(Error::custom)
                .map(Event::StreamOfflineEvent),
            "channel.update" => serde_json::from_str(event.event.get())
                .map_err(Error::custom)
                .map(Event::ChannelUpdateEvent),
            ty => Err(Error::custom(format!("unknown event type {ty}"))),
        }
    }
}

#[test]
fn de() {
    use serde_json::from_str;

    from_str::<Event>(
        r#"
        {
            "subscription": {
                "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
                "type": "stream.online",
                "version": "1",
                "status": "enabled",
                "cost": 0,
                "condition": {
                    "broadcaster_user_id": "1337"
                },
                 "transport": {
                    "method": "webhook",
                    "callback": "https://example.com/webhooks/callback"
                },
                "created_at": "2019-11-16T10:11:12.634234626Z"
            },
            "event": {
                "id": "9001",
                "broadcaster_user_id": "1337",
                "broadcaster_user_login": "cool_user",
                "broadcaster_user_name": "Cool_User",
                "type": "live",
                "started_at": "2020-10-11T10:11:12.123Z"
            }
        }
        "#,
    )
    .unwrap();

    from_str::<Event>(
        r#"
        {
            "subscription": {
                "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
                "type": "channel.update",
                "version": "2",
                "status": "enabled",
                "cost": 0,
                "condition": {
                   "broadcaster_user_id": "1337"
                },
                 "transport": {
                    "method": "webhook",
                    "callback": "https://example.com/webhooks/callback"
                },
                "created_at": "2023-06-29T17:20:33.860897266Z"
            },
            "event": {
                "broadcaster_user_id": "1337",
                "broadcaster_user_login": "cool_user",
                "broadcaster_user_name": "Cool_User",
                "title": "Best Stream Ever",
                "language": "en",
                "category_id": "12453",
                "category_name": "Grand Theft Auto",
                "content_classification_labels": [ "MatureGame" ]
            }
        }
        "#,
    )
    .unwrap();

    from_str::<Event>(
        r#"
        {
            "subscription": {
                "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
                "type": "stream.offline",
                "version": "1",
                "status": "enabled",
                "cost": 0,
                "condition": {
                    "broadcaster_user_id": "1337"
                },
                "created_at": "2019-11-16T10:11:12.634234626Z",
                 "transport": {
                    "method": "webhook",
                    "callback": "https://example.com/webhooks/callback"
                }
            },
            "event": {
                "broadcaster_user_id": "1337",
                "broadcaster_user_login": "cool_user",
                "broadcaster_user_name": "Cool_User"
            }
        }
        "#,
    )
    .unwrap();

    from_str::<Vec<CreatedSubscription>>(
        r#"
    [
        {
            "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
            "type": "channel.update",
            "version": "2",
            "status": "enabled",
            "cost": 0,
            "condition": {
               "broadcaster_user_id": "1337"
            },
             "transport": {
                "method": "webhook",
                "callback": "https://example.com/webhooks/callback"
            },
            "created_at": "2023-06-29T17:20:33.860897266Z"
        },
          {
            "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
            "type": "stream.offline",
            "version": "1",
            "status": "enabled",
            "cost": 0,
            "condition": {
                "broadcaster_user_id": "1337"
            },
            "created_at": "2019-11-16T10:11:12.634234626Z",
             "transport": {
                "method": "webhook",
                "callback": "https://example.com/webhooks/callback"
            }
        },
        {
            "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
            "type": "stream.online",
            "version": "1",
            "status": "enabled",
            "cost": 0,
            "condition": {
                "broadcaster_user_id": "1337"
            },
             "transport": {
                "method": "webhook",
                "callback": "https://example.com/webhooks/callback"
            },
            "created_at": "2019-11-16T10:11:12.634234626Z"
        }
    ]
    "#,
    )
    .unwrap();
}
