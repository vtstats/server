use metrics::{histogram, increment_counter};
use sqlx::Result;
use std::{future::Future, time::Instant};
use tracing::Instrument;

#[inline(always)]
pub async fn instrument<T>(
    operation: &'static str,
    table: &'static str,
    query: impl Future<Output = Result<T>>,
) -> Result<T> {
    let span = tracing::info_span!(
        "Database Query",
        "message" = format!("{operation} {table}"),
        "span.kind" = "client",
        "db.operation" = operation,
        "db.sql.table" = table,
    );

    async move {
        let start = Instant::now();

        let result = query.await;

        histogram!(
            "postgres_queries_elapsed_seconds",
            start.elapsed(),
            "operation" => operation,
            "table" => table,
        );
        increment_counter!(
            "postgres_queries_count",
            "operation" => operation,
            "table" => table,
        );

        // TODO: use `inspect_err` once stable
        if let Err(err) = &result {
            tracing::error!(exception.stacktrace = ?err, message= %err);
        }

        result
    }
    .instrument(span)
    .await
}
