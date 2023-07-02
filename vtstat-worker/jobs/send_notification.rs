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

use integration_discord::message::{
    CreateMessageRequest, EditMessageRequest, Embed, EmbedAuthor, EmbedField, EmbedFooter,
    EmbedImage, EmbedThumbnail,
};

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

    let embeds = vec![build_discord_embed(stream, &payload.vtuber_id, &pool).await?];

    for subscription in subscriptions {
        let previous_notification = ListNotificationsQuery {
            subscription_id: subscription.subscription_id,
            stream_id: stream.stream_id,
        }
        .execute(pool)
        .await?;

        match previous_notification {
            Some(notification) => {
                EditMessageRequest {
                    channel_id: subscription.payload.channel_id,
                    content: String::new(),
                    message_id: notification.payload.message_id,
                    embeds: embeds.clone(),
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
                    content: String::new(),
                    embeds: embeds.clone(),
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

pub async fn build_discord_embed(
    stream: &Stream,
    vtuber_id: &str,
    pool: &PgPool,
) -> anyhow::Result<Embed> {
    let vtubers = ListVtubersQuery.execute(pool).await?;

    let vtuber = vtubers.iter().find(|vtb| vtb.vtuber_id == vtuber_id);

    let mut embed = Embed::default();

    embed.title = Some(stream.title.clone());
    embed.color = Some(match stream.status {
        StreamStatus::Scheduled => 0x009688,
        StreamStatus::Live => 0xD81B60,
        StreamStatus::Ended => 0x3F51B5,
    });
    embed.url = Some(format!("https://youtu.be/{}", stream.platform_id));
    embed.thumbnail = Some(EmbedThumbnail {
        url: format!("https://holo.poi.cat/assets/thumbnail/{vtuber_id}.jpg"),
    });
    if let Some(vtuber) = vtuber {
        embed.author = Some(EmbedAuthor {
            name: vtuber.native_name.clone(),
            url: format!("https://holo.poi.cat/vtuber/{vtuber_id}"),
        })
    }

    match (stream.schedule_time, stream.start_time, stream.end_time) {
        (Some(schedule), None, None) => embed.fields.push(EmbedField {
            name: "Time".into(),
            value: format!("<t:{ts}>, <t:{ts}:R>", ts = schedule.timestamp()),
            inline: true,
        }),
        (_, Some(start), None) => embed.fields.push(EmbedField {
            name: "Time".into(),
            value: format!("<t:{ts}>, <t:{ts}:R> ⇨", ts = start.timestamp()),
            inline: true,
        }),
        (_, None, Some(end)) => embed.fields.push(EmbedField {
            name: "Time".into(),
            value: format!("⇨ <t:{ts}>", ts = end.timestamp()),
            inline: true,
        }),
        (_, Some(start), Some(end)) => {
            embed.fields.push(EmbedField {
                name: "Time".into(),
                value: format!("<t:{}> ⇨ <t:{}>", start.timestamp(), end.timestamp()),
                inline: true,
            });

            let minutes = (end - start).num_minutes();
            let mut value = String::new();
            if minutes > 60 {
                value.push_str(&format!("{} hours ", (minutes / 60) | 0));
            }
            value.push_str(&format!("{} minutes", minutes % 60));
            embed.fields.push(EmbedField {
                name: "Duration".into(),
                value,
                inline: true,
            });
        }
        (None, None, None) => {}
    }
    if let Some(thumbnail_url) = &stream.thumbnail_url {
        embed.image = Some(EmbedImage {
            url: thumbnail_url.clone(),
            height: Some(720),
            width: Some(1280),
        })
    }
    embed.timestamp = Some(stream.updated_at.to_rfc3339());
    embed.footer = Some(EmbedFooter {
        text: (concat!("vtstat ", env!("CARGO_PKG_VERSION"))).into(),
    });

    Ok(embed)
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
