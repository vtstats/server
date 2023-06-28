use anyhow::bail;
use std::fmt::Write;
use vtstat_database::{
    channels::ListChannelsQuery,
    jobs::SendNotificationJobPayload,
    streams::{ListYouTubeStreamsQuery, Stream, StreamStatus},
    subscriptions::{
        InsertNotificationQuery, ListDiscordSubscriptionQuery, ListNotificationsQuery,
        NotificationPayload, UpdateNotificationQuery,
    },
    vtubers::ListVtubersQuery,
    PgPool,
};
use vtstat_request::RequestHub;

use integration_discord::message::{CreateMessageRequest, EditMessageRequest};

use super::JobResult;

pub async fn execute(
    pool: &PgPool,
    hub: RequestHub,
    payload: SendNotificationJobPayload,
) -> anyhow::Result<JobResult> {
    if payload.platform != "discord" {
        return Ok(JobResult::Completed);
    }

    let subscriptions = ListDiscordSubscriptionQuery::ByVtuberId(payload.vtuber_id.clone())
        .execute(pool)
        .await?;

    if subscriptions.is_empty() {
        return Ok(JobResult::Completed);
    }

    let streams = ListYouTubeStreamsQuery {
        platform_ids: &[payload.stream_platform_id],
        ..Default::default()
    }
    .execute(pool)
    .await?;

    let Some(stream) = streams.first() else {
        return Ok(JobResult::Completed);
    };

    for subscription in subscriptions {
        let previous_notification = ListNotificationsQuery {
            subscription_id: subscription.subscription_id,
        }
        .execute(pool)
        .await?;

        match previous_notification {
            Some(notification) => {
                EditMessageRequest {
                    channel_id: subscription.payload.channel_id,
                    content: format!("{:?}", &stream),
                    message_id: notification.payload.message_id,
                }
                .send(&hub.client)
                .await?;

                UpdateNotificationQuery {
                    notification_id: notification.notification_id,
                }
                .execute(pool)
                .await?;
            }
            None => {
                let msg_id = CreateMessageRequest {
                    channel_id: subscription.payload.channel_id,
                    content: format!("{:?}", &stream),
                }
                .send(&hub.client)
                .await?;

                InsertNotificationQuery {
                    subscription_id: subscription.subscription_id,
                    payload: NotificationPayload {
                        vtuber_id: payload.vtuber_id.clone(),
                        stream_id: stream.stream_id,
                        message_id: msg_id,
                    },
                }
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(JobResult::Completed)
}

pub async fn build_telegram_message(stream: &Stream, pool: &PgPool) -> anyhow::Result<String> {
    let channels = ListChannelsQuery {
        platform: "youtube",
    }
    .execute(pool)
    .await?;

    let vtubers = ListVtubersQuery.execute(pool).await?;

    let vtuber = channels
        .iter()
        .find(|ch| ch.platform_id == stream.platform_channel_id)
        .and_then(|ch| vtubers.iter().find(|vtb| vtb.vtuber_id == ch.vtuber_id));

    let Some(vtuber) = vtuber else {
        bail!("Stream not found");
    };

    let mut buf = String::new();
    let _ = writeln!(
        buf,
        r#"<a href="http://youtu.be/{}">{}</a>"#,
        stream.platform_id, stream.title,
    );
    let _ = writeln!(
        buf,
        r#"from <a href="https://www.youtube.com/channel/{}">{}</a>"#,
        stream.platform_channel_id, vtuber.native_name,
    );
    match stream.status {
        StreamStatus::Scheduled => {
            if let Some(time) = stream.schedule_time {
                let _ = writeln!(buf, "scheduled at {}", time.to_rfc3339());
            }
        }
        StreamStatus::Live => {
            if let Some(time) = stream.start_time {
                let _ = writeln!(buf, "started at {}", time.to_rfc3339());
            }
        }
        StreamStatus::Ended => {
            if let Some(time) = stream.end_time {
                let _ = writeln!(buf, "ended at {}", time.to_rfc3339());
            }
        }
    }
    let _ = write!(buf, "#youtube #{}", vtuber.vtuber_id);

    Ok(buf)
}
