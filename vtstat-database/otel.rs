use sqlx::Result;
use std::future::Future;
use tracing::{
    field::{debug, display, Empty},
    Instrument, Span,
};

#[inline(always)]
pub async fn instrument<T>(
    operation: &'static str,
    table: &'static str,
    query: impl Future<Output = Result<T>>,
) -> Result<T> {
    let database = "holostats";

    let span = tracing::info_span!(
        "Query",
        name = format!("{operation} {database}.{table}"),
        span.kind = "client",
        //// database
        db.name = database,
        db.system = "postgresql",
        db.operation = operation,
        db.sql.table = table,
        //// error
        error.message = Empty,
        error.cause_chain = Empty,
    );

    async move {
        let result = query.await;

        // TODO: use `inspect_err` once stable
        if let Err(err) = &result {
            Span::current()
                .record("otel.status_code", "ERROR")
                .record("error.message", display(err))
                .record("error.cause_chain", debug(err));
        }

        result
    }
    .instrument(span)
    .await
}
