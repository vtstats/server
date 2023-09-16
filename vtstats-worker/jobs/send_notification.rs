use anyhow::bail;
use reqwest::Client;
use std::{fmt::Write, vec};

use integration_discord::message::{
    CreateMessageRequest, EditMessageRequest, Embed, EmbedAuthor, EmbedField, EmbedFooter,
    EmbedImage, EmbedThumbnail, MessageReference,
};
use vtstats_database::{
    channels::Platform,
    streams::{get_stream_by_id, Stream, StreamStatus},
    subscriptions::{
        list_discord_subscription_and_notification_by_vtuber_id, update_discord_notification,
        DiscordSubscriptionAndNotification, InsertNotificationQuery, NotificationPayload,
    },
    vtubers::find_vtuber,
    PgPool,
};

use super::JobResult;

pub async fn execute(pool: &PgPool, client: Client, stream_id: i32) -> anyhow::Result<JobResult> {
    let Some(stream) = get_stream_by_id(stream_id, pool).await? else {
        tracing::warn!("Can't find stream with id: {}", stream_id);
        return Ok(JobResult::Completed);
    };

    if stream.platform != Platform::Youtube {
        tracing::warn!("Can't find stream with id: {}", stream_id);
        return Ok(JobResult::Completed);
    }

    let subscriptions = list_discord_subscription_and_notification_by_vtuber_id(
        stream.vtuber_id.clone(),
        stream.stream_id,
        pool,
    )
    .await?;

    if subscriptions.is_empty() {
        return Ok(JobResult::Completed);
    }

    let embeds = vec![build_discord_embed(&stream, &stream.vtuber_id, pool).await?];

    for item in subscriptions {
        let result = send_discord_notification(&item, &stream, embeds.clone(), pool, &client).await;
        if let Err(err) = result {
            tracing::error!(
                "Failed to send discord notification guild_id={} vtuber_id={} channel_id={} stream_id={}",
                item.subscription_payload.guild_id,
                item.subscription_payload.vtuber_id,
                item.subscription_payload.channel_id,
                stream.stream_id,
            );
            tracing::error!("Error: {:?}", err);
        }
    }

    Ok(JobResult::Completed)
}

async fn send_discord_notification(
    item: &DiscordSubscriptionAndNotification,
    stream: &Stream,
    embeds: Vec<Embed>,
    pool: &PgPool,
    client: &Client,
) -> anyhow::Result<()> {
    let subscription = &item.subscription_payload;
    let subscription_id = item.subscription_id;
    let notification = &item.notification_payload;
    let notification_id = item.notification_id;

    match (notification, notification_id) {
        (Some(notification), Some(notification_id)) => {
            EditMessageRequest {
                channel_id: subscription.channel_id.clone(),
                content: String::new(),
                message_id: notification.message_id.clone(),
                embeds,
            }
            .send(client)
            .await?;

            let mut start_message_id = notification.start_message_id.clone();
            let mut end_message_id = notification.end_message_id.clone();

            if stream.status == StreamStatus::Live && start_message_id.is_none() {
                let message_id = CreateMessageRequest {
                    channel_id: subscription.channel_id.clone(),
                    content: "Stream has started".into(),
                    embeds: vec![],
                    message_reference: Some(MessageReference {
                        message_id: notification.message_id.clone(),
                        fail_if_not_exists: false,
                    }),
                }
                .send(client)
                .await?;
                start_message_id = Some(message_id);
            }

            if stream.status == StreamStatus::Ended && end_message_id.is_none() {
                let message_id = CreateMessageRequest {
                    channel_id: subscription.channel_id.clone(),
                    content: "Stream has ended".into(),
                    embeds: vec![],
                    message_reference: Some(MessageReference {
                        message_id: notification.message_id.clone(),
                        fail_if_not_exists: false,
                    }),
                }
                .send(client)
                .await?;
                end_message_id = Some(message_id);
            }

            update_discord_notification(
                notification_id,
                notification.message_id.clone(),
                start_message_id,
                end_message_id,
                pool,
            )
            .await?;
        }
        _ => {
            let message_id = CreateMessageRequest {
                channel_id: subscription.channel_id.clone(),
                content: String::new(),
                embeds: embeds.clone(),
                message_reference: None,
            }
            .send(client)
            .await?;

            InsertNotificationQuery {
                subscription_id,
                payload: NotificationPayload {
                    vtuber_id: stream.vtuber_id.clone(),
                    stream_id: stream.stream_id,
                    message_id,
                    start_message_id: None,
                    end_message_id: None,
                },
            }
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

async fn build_discord_embed(
    stream: &Stream,
    vtuber_id: &str,
    pool: &PgPool,
) -> anyhow::Result<Embed> {
    let vtuber = find_vtuber(vtuber_id, pool).await?;

    let color = match stream.status {
        StreamStatus::Scheduled => 0x009688,
        StreamStatus::Live => 0xD81B60,
        StreamStatus::Ended => 0x3F51B5,
    };
    let thumbnail = vtuber
        .as_ref()
        .and_then(|v| v.thumbnail_url.as_ref())
        .map(|url| EmbedThumbnail { url: url.clone() });
    let author = vtuber.map(|v| {
        let mut name = v.native_name.to_string();
        match (&v.english_name, &v.japanese_name) {
            (None, Some(n)) | (Some(n), None) => {
                if n != &v.native_name {
                    let _ = write!(&mut name, " / {n}");
                }
            }
            (Some(n1), Some(n2)) => {
                if n1 != &v.native_name {
                    let _ = write!(&mut name, " / {n1}");
                }
                if n1 != n2 && n2 != &v.native_name {
                    let _ = write!(&mut name, " / {n2}");
                }
            }
            _ => {}
        }

        EmbedAuthor {
            name,
            url: format!("https://vt.poi.cat/vtuber/{vtuber_id}"),
        }
    });

    let fields = match (stream.schedule_time, stream.start_time, stream.end_time) {
        (Some(schedule), None, None) => vec![EmbedField {
            name: "Schedule".into(),
            value: format!("<t:{ts}>, <t:{ts}:R>", ts = schedule.timestamp()),
            inline: true,
        }],
        (_, Some(start), None) => vec![EmbedField {
            name: "Start".into(),
            value: format!("<t:{ts}>, <t:{ts}:R>", ts = start.timestamp()),
            inline: true,
        }],
        (_, None, Some(end)) => vec![EmbedField {
            name: "End".into(),
            value: format!("<t:{}>", end.timestamp()),
            inline: true,
        }],
        (_, Some(start), Some(end)) => {
            let total_minutes = (end - start).num_minutes();
            let hours = total_minutes / 60;
            let minutes = total_minutes % 60;

            let mut value = String::new();
            if hours > 0 {
                value.push_str(&hours.to_string());
                value.push_str(if hours > 1 { " hours" } else { " hour" });
            }
            if minutes > 0 {
                if hours > 0 {
                    value.push(' ');
                }
                value.push_str(&minutes.to_string());
                value.push_str(if minutes > 1 { " minutes" } else { " minute" });
            }
            vec![
                EmbedField {
                    name: "Start".into(),
                    value: format!("<t:{}>", start.timestamp()),
                    inline: true,
                },
                EmbedField {
                    name: "End".into(),
                    value: format!("<t:{}>", end.timestamp()),
                    inline: true,
                },
                EmbedField {
                    name: "Duration".into(),
                    value,
                    inline: true,
                },
            ]
        }
        (None, None, None) => vec![],
    };

    let image = stream.thumbnail_url.as_ref().map(|t| EmbedImage {
        url: t.clone(),
        height: Some(720),
        width: Some(1280),
    });

    let footer = EmbedFooter {
        text: (concat!("vtstat ", env!("CARGO_PKG_VERSION"))).into(),
    };

    Ok(Embed {
        timestamp: Some(stream.updated_at.to_rfc3339()),
        title: Some(stream.title.clone()),
        url: Some(format!("https://youtu.be/{}", stream.platform_id)),
        color: Some(color),
        description: None,
        footer: Some(footer),
        author,
        image,
        thumbnail,
        fields,
    })
}

pub async fn _build_telegram_message(
    stream: &Stream,
    vtuber_id: &str,
    pool: &PgPool,
) -> anyhow::Result<String> {
    let vtuber = find_vtuber(vtuber_id, pool).await?;

    let Some(vtuber) = vtuber else {
        bail!("Stream not found");
    };

    let mut buf = String::new();
    let _ = writeln!(
        buf,
        r#"<a href="http://youtu.be/{}">{}</a>"#,
        stream.platform_id, stream.title,
    );
    // let _ = writeln!(
    //     buf,
    //     r#"from <a href="https://www.youtube.com/channel/{}">{}</a>"#,
    //     stream.platform_channel_id, vtuber.native_name,
    // );
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
