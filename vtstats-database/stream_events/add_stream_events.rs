use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, types::Json, PgPool, Postgres, QueryBuilder, Result};

use super::StreamEventValue;

pub async fn add_stream_events(
    stream_id: i32,
    rows: Vec<(DateTime<Utc>, StreamEventValue)>,
    pool: &PgPool,
) -> Result<PgQueryResult> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO stream_events (stream_id, time, kind, value) ");

    query_builder.push_values(rows.into_iter(), |mut b, (time, value)| {
        b.push_bind(stream_id)
            .push_bind(time)
            .push_bind(value.kind())
            .push_bind(Json(value));
    });

    let query = query_builder.build().execute(pool);

    crate::otel::execute_query!("INSERT", "stream_events", query)
}

// TODO: add unit tests
