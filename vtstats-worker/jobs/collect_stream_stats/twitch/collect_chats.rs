use chrono::{DateTime, Duration, DurationRound, Utc};
use integration_twitch::{connect_chat_room, read_live_chat_message, LiveChatMessage};
use vtstats_database::{
    stream_events::{
        add_stream_events, StreamEvent, StreamEventKind, StreamEventValue, TwitchCheering,
        TwitchHyperChat,
    },
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow},
    PgPool,
};

pub async fn collect_chats(stream_id: i32, login: &str, pool: &PgPool) -> anyhow::Result<()> {
    let mut tcp = connect_chat_room(login.to_string()).await?;

    let mut time: Option<DateTime<Utc>> = None;
    let mut count = 0;
    let mut from_member_count = 0;
    let mut events: Option<StreamEvent> = None;

    loop {
        let msg = read_live_chat_message(&mut tcp).await?;

        let (timestamp, from_subscriber) = match msg {
            LiveChatMessage::HyperChat {
                timestamp,
                amount,
                level,
                currency_code,
                author_username,
                badges,
                text,
            } => {
                events = Some(StreamEvent {
                    kind: StreamEventKind::TwitchHyperChat,
                    time: timestamp,
                    value: StreamEventValue::TwitchHyperChat(TwitchHyperChat {
                        amount,
                        author_username,
                        badges,
                        currency_code,
                        level,
                        message: text,
                    }),
                });
                (timestamp, false)
            }
            LiveChatMessage::Cheering {
                timestamp,
                author_username,
                badges,
                bits,
                text,
            } => {
                events = Some(StreamEvent {
                    kind: StreamEventKind::TwitchCheering,
                    time: timestamp,
                    value: StreamEventValue::TwitchCheering(TwitchCheering {
                        badges,
                        bits,
                        message: text,
                        author_username,
                    }),
                });
                (timestamp, false)
            }
            LiveChatMessage::Text { timestamp, .. } => (timestamp, false),
            LiveChatMessage::Subscriber { timestamp, .. } => (timestamp, true),
        };

        let timestamp = timestamp.duration_trunc(Duration::seconds(15)).unwrap();

        match time {
            Some(time) if time != timestamp => {
                AddStreamChatStatsQuery {
                    stream_id,
                    rows: vec![AddStreamChatStatsRow {
                        time,
                        count,
                        from_member_count,
                    }],
                }
                .execute(pool)
                .await?;
                count = 0;
                from_member_count = 0;
            }
            _ => {
                count += 1;
                if from_subscriber {
                    from_member_count += 1;
                }
            }
        }

        time = Some(timestamp);

        if let Some(event) = events.take() {
            add_stream_events(stream_id, vec![(event.time, event.value)], pool).await?;
        }
    }
}
