macro_rules! execute_query {
    ($operation:expr, $table:expr, $query:expr) => {{
        use tracing::Instrument;
        use std::time::Instant;

        let span = tracing::info_span!(
            "Database Query",
            "message" = format!("{} {}", $operation, $table),
            "span.kind" = "client",
            "db.operation" = $operation,
            "db.sql.table" = $table,
        );

        async move {
            let start = Instant::now();

            let result = $query.await;

            metrics::histogram!(
                "postgres_queries_elapsed_seconds",
                start.elapsed(),
                "operation" => $operation,
                "table" => $table,
            );

            // TODO: use `inspect_err` once stable
            if let Err(err) = &result {
                tracing::error!(exception.stacktrace = ?err, message= %err);
            }

            result
        }
        .instrument(span)
        .await
    }};
}

pub(crate) use execute_query;
