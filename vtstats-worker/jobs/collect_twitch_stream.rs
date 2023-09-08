use chrono::{Duration, DurationRound};
use integration_twitch::{
    connect_chat_room, gql::stream_metadata, read_live_chat_message, LiveChatMessage,
};
use reqwest::Client;
use vtstats_database::{
    jobs::CollectTwitchStreamMetadataJobPayload,
    stream_stats::{AddStreamChatStatsQuery, AddStreamChatStatsRow},
    streams::{find_stream, update_stream_title, StreamStatus},
    PgPool,
};

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    client: Client,
    payload: CollectTwitchStreamMetadataJobPayload,
) -> anyhow::Result<JobResult> {
    let metadata = stream_metadata(&payload.platform_channel_login, &client).await?;

    let Some(stream) = metadata.data.user.stream else {
        return Ok(JobResult::Completed);
    };

    if stream.id != payload.platform_stream_id {
        return Ok(JobResult::Completed);
    }

    let mut title = None;
    if matches!(metadata.data.user.last_broadcast.id, Some(id) if id == stream.id) {
        title = metadata.data.user.last_broadcast.title
    }

    if let Some(title) = title {
        update_stream_title(payload.stream_id, title, pool).await?;
    }

    tokio::select! {
        _ = check_if_stream_online(payload.stream_id, pool) => {},
        _ = collect_live_chat_message(payload.stream_id, payload.platform_channel_login, pool) => {},
    };

    Ok(JobResult::Completed)
}

async fn collect_live_chat_message(
    stream_id: i32,
    login: String,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut tcp = connect_chat_room(login).await?;

    loop {
        let msg = read_live_chat_message(&mut tcp).await?;

        let (timestamp, from_subscriber) = match msg {
            LiveChatMessage::HyperChat {
                timestamp,
                amount,
                level,
                currency,
                ..
            } => {
                tracing::warn!("amount:{amount}, level:{level}, currency:{currency}");
                (timestamp, false)
            }
            LiveChatMessage::Text { timestamp, .. } => (timestamp, false),
            LiveChatMessage::Subscriber { timestamp, .. } => (timestamp, true),
        };

        let timestamp = timestamp.duration_trunc(Duration::seconds(15)).unwrap();

        AddStreamChatStatsQuery {
            stream_id,
            rows: vec![AddStreamChatStatsRow {
                time: timestamp,
                count: 1,
                from_member_count: if from_subscriber { 1 } else { 0 },
            }],
        }
        .execute(pool)
        .await?;
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
