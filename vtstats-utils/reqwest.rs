use backon::{ExponentialBuilder, Retryable};
use futures::TryFutureExt;
use metrics::histogram;
use reqwest::{Client, ClientBuilder, Proxy, Request, Response, Result};
use std::{
    env,
    time::{Duration, Instant},
};

pub fn new() -> Result<Client> {
    let mut builder = ClientBuilder::new()
        .http1_only()
        .brotli(true)
        .deflate(true)
        .gzip(true);

    if let Ok(proxy) = env::var("ALL_PROXY") {
        builder = builder.proxy(Proxy::all(proxy)?);
    }

    builder.build()
}

#[macro_export]
macro_rules! send_request {
    (@internal, $client:expr, $req:expr, $path:expr) => {{
        use ::vtstats_utils::reqwest::instrument_send;
        use tracing::{field::Empty, Instrument};

        let method = $req.method().as_str();
        let request_content_length = $req.body().and_then(|b| b.as_bytes()).map(|b| b.len());

        // create a span inside macro, and file!, line!
        // can show the correct result
        let span = tracing::info_span!(
            "Http Client",
            "message" = format!("{} {}", method, $path),
            "span.kind" = "client",
            "http.req.method" = method,
            "http.req.host" = $req.url().domain(),
            "http.req.path" = $path,
            "http.req.content_length" = request_content_length,
            "http.res.status_code" = Empty,
            "http.res.content_length" = Empty,
        );

        instrument_send($client, $req, $path)
            .instrument(span)
            .await
    }};

    ($req:expr) => {{
        let (client, req) = $req.build_split();
        let req = req?;
        let path = req.url().path().to_string();
        send_request!(@internal, &client, req, path)
    }};

    ($req:expr, $path:literal) => {{
        let (client, req) = $req.build_split();
        let req = req?;
        send_request!(@internal, &client, req, $path.to_string())
    }};
}

#[inline(always)]
pub async fn instrument_send(client: &Client, req: Request, path: String) -> Result<Response> {
    let future_fn = || {
        let req = req.try_clone().expect("request body must not be stream");
        execute_with_metrics(req, client, path.clone())
    };

    let retry_builder = ExponentialBuilder::default()
        .with_min_delay(Duration::from_secs(1))
        .with_factor(1.5)
        .with_max_times(10);

    future_fn
        .retry(&retry_builder)
        // TODO: use `inspect_err` once stable
        .map_err(|err| {
            tracing::error!(exception.stacktrace = ?err, message= %err);
            err
        })
        .await
}

async fn execute_with_metrics(req: Request, client: &Client, path: String) -> Result<Response> {
    let start = Instant::now();
    let method = req.method().as_str().to_string();
    let url = req.url();
    let host = url.domain().map(|h| h.to_string()).unwrap_or_default();

    client
        .execute(req)
        .await
        // TODO: use `inspect` once stable
        .map(|res| {
            let status_code = res.status().as_str().to_string();
            histogram!(
                "http_client_requests_elapsed_seconds",
                start.elapsed(),
                "method" => method.clone(),
                "path" => path.clone(),
                "host" => host.clone(),
                "status_code" => status_code,
            );
            res
        })
        .and_then(|r| r.error_for_status())
}
