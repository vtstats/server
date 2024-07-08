use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use vtstats_database::{
    channel_stats_summary::{self, ChannelStatsKind},
    channels::{CreateChannel, Platform},
};
use vtstats_utils::context::AppContext;

use crate::error::ApiResult;

use super::ActionResponse;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub vtuber_id: String,
    pub kind: Option<String>,
    pub platform: Platform,
    pub platform_id: String,
}

pub async fn create_channel(
    State(state): State<AppContext>,
    Json(payload): Json<Payload>,
) -> ApiResult<impl IntoResponse> {
    let mut tx = state.pool.begin().await?;

    let channel_id = CreateChannel {
        platform: payload.platform,
        platform_id: payload.platform_id,
        vtuber_id: payload.vtuber_id.clone(),
        kind: payload.kind,
    }
    .execute(&mut *tx)
    .await?;

    match payload.platform {
        Platform::Youtube => {
            channel_stats_summary::create(channel_id, ChannelStatsKind::View, &mut *tx).await?;
            channel_stats_summary::create(channel_id, ChannelStatsKind::Subscriber, &mut *tx)
                .await?;
            channel_stats_summary::create(channel_id, ChannelStatsKind::Revenue, &mut *tx).await?;
        }
        Platform::Bilibili => {
            channel_stats_summary::create(channel_id, ChannelStatsKind::View, &mut *tx).await?;
            channel_stats_summary::create(channel_id, ChannelStatsKind::Subscriber, &mut *tx)
                .await?;
        }
        Platform::Twitch => {
            channel_stats_summary::create(channel_id, ChannelStatsKind::Subscriber, &mut *tx)
                .await?;
            channel_stats_summary::create(channel_id, ChannelStatsKind::Revenue, &mut *tx).await?;
        }
    }

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(ActionResponse {
            msg: format!("Channel {} was created.", channel_id),
        }),
    ))
}
