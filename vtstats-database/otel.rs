use metrics::histogram;
use sqlx::Result;
use std::{future::Future, time::Instant};

macro_rules! execute_query {
    ($operation:expr, $table:expr, $query:expr) => {{
        use tracing::Instrument;

        let span = tracing::info_span!(
            "Database Query",
            "message" = format!("{} {}", $operation, $table),
            "span.kind" = "client",
            "db.operation" = $operation,
            "db.sql.table" = $table,
        );

        crate::otel::instrument_($operation, $table, $query)
            .instrument(span)
            .await
    }};
}

pub(crate) use execute_query;

#[inline(always)]
pub async fn instrument_<T>(
    operation: &'static str,
    table: &'static str,
    query: impl Future<Output = Result<T>>,
) -> Result<T> {
    let start = Instant::now();

    let result = query.await;

    histogram!(
        "postgres_queries_elapsed_seconds",
        start.elapsed(),
        "operation" => operation,
        "table" => table,
    );

    // TODO: use `inspect_err` once stable
    if let Err(err) = &result {
        tracing::error!(exception.stacktrace = ?err, message= %err);
    }

    result
}
