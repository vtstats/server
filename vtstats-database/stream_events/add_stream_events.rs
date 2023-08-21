use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, types::Json, PgPool, Postgres, QueryBuilder, Result};

use super::{StreamEventKind, StreamEventValue};

pub async fn add_stream_events(
    stream_id: i32,
    rows: Vec<(DateTime<Utc>, StreamEventValue)>,
    pool: &PgPool,
) -> Result<PgQueryResult> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO stream_events (stream_id, time, kind, value) ");

    query_builder.push_values(rows.into_iter(), |mut b, (time, value)| {
        let kind = match value {
            StreamEventValue::YoutubeSuperChat { .. } => StreamEventKind::YoutubeSuperChat,
            StreamEventValue::YoutubeSuperSticker { .. } => StreamEventKind::YoutubeSuperSticker,
            StreamEventValue::YoutubeNewMember { .. } => StreamEventKind::YoutubeNewMember,
            StreamEventValue::YoutubeMemberMilestone { .. } => {
                StreamEventKind::YoutubeMemberMilestone
            }
        };

        b.push_bind(stream_id)
            .push_bind(time)
            .push_bind(kind)
            .push_bind(Json(value));
    });

    let query = query_builder.build().execute(pool);

    crate::otel::instrument("INSERT", "stream_events", query).await
}

// TODO: add unit tests
