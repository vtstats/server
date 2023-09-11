use chrono::{DateTime, Duration, DurationRound, Utc};
use reqwest::Client;
use tokio::signal::unix;

use integration_twitch::{
    connect_chat_room, gql::use_view_count, read_live_chat_message, LiveChatMessage,
};
use vtstats_database::{
    jobs::CollectTwitchStreamMetadataJobPayload,
    stream_events::{
        add_stream_events, StreamEvent, StreamEventKind, StreamEventValue, TwitchCheering,
        TwitchHyperChat,
    },
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow, AddStreamViewerStatsQuery},
    streams::{find_stream, StreamStatus},
    PgPool,
};

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    client: Client,
    payload: CollectTwitchStreamMetadataJobPayload,
) -> anyhow::Result<JobResult> {
    let (Ok(mut sigint), Ok(mut sigterm)) = (
        unix::signal(unix::SignalKind::interrupt()),
        unix::signal(unix::SignalKind::terminate()),
    ) else {
        anyhow::bail!("Failed to listen unix signal")
    };

    tokio::select! {
        _ = check_if_stream_online(payload.stream_id, pool) => {
            Ok(JobResult::Completed)
        },
        _ = collect_stream_chats(payload.stream_id, &payload.platform_channel_login, pool) => {
            Ok(JobResult::Completed)
        },
        _ = collect_stream_viewers(payload.stream_id, &payload.platform_channel_login, &client, pool) => {
            Ok(JobResult::Completed)
        },
        _ = sigint.recv() => {
            Ok(JobResult::Next { run: Utc::now()  })
        },
        _ = sigterm.recv() => {
            Ok(JobResult::Next { run: Utc::now()  })
        },
    }
}

async fn collect_stream_viewers(
    stream_id: i32,
    login: &str,
    client: &Client,
    pool: &PgPool,
) -> anyhow::Result<()> {
    loop {
        let res = use_view_count(login.to_string(), client).await?;

        if let Some(stream) = res.data.user.stream {
            AddStreamViewerStatsQuery {
                stream_id,
                time: Utc::now().duration_trunc(Duration::seconds(15)).unwrap(),
                count: stream.viewers_count,
            }
            .execute(pool)
            .await?;
        }

        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}

async fn collect_stream_chats(stream_id: i32, login: &str, pool: &PgPool) -> anyhow::Result<()> {
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

async fn check_if_stream_online(stream_id: i32, pool: &PgPool) -> anyhow::Result<()> {
    loop {
        let stream = find_stream(stream_id, pool).await?;

        if !matches!(stream, Some(stream) if stream.status == StreamStatus::Live) {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
