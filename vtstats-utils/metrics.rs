use backon::{ExponentialBuilder, Retryable};
use futures::TryFutureExt;
use metrics::{histogram, increment_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use reqwest::Request;
use std::{
    env,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tracing::{field::Empty, Instrument};

pub fn install() {
    let mut builder = PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .set_buckets(&[
            0., 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1., 2.5, 5., 7.5, 10.,
        ])
        .unwrap();

    if let Some(address) = env::var("METRICS_ADDRESS")
        .ok()
        .and_then(|add| add.parse::<SocketAddr>().ok())
    {
        builder = builder.with_http_listener(address);
    }

    builder.install().expect("failed to install recorder");
}

pub async fn instrument_send(
    client: &reqwest::Client,
    req: reqwest::RequestBuilder,
) -> reqwest::Result<reqwest::Response> {
    let req = req.build()?;

    let method = req.method().as_str().to_string();
    let url = req.url();
    let path = url.path().to_string();
    let host = url.domain().map(|h| h.to_string()).unwrap_or_default();
    let request_content_length = req.body().and_then(|b| b.as_bytes()).map(|b| b.len());
    let name = format!("{method} {path}");

    let span = tracing::info_span!(
        "Http Client",
        "message" = name,
        "span.kind" = "client",
        "http.req.method" = &method,
        "http.req.host" = &host,
        "http.req.path" = &path,
        "http.req.content_length" = request_content_length,
        "http.res.status_code" = Empty,
        "http.res.content_length" = Empty,
    );

    let future_fn = || {
        let req = req.try_clone().expect("request body must not be stream");
        execute_with_metrics(req, client)
    };

    let retry_builder = ExponentialBuilder::default()
        .with_min_delay(Duration::from_secs(1))
        .with_factor(1.25)
        .with_max_times(5);

    future_fn
        .retry(&retry_builder)
        // TODO: use `inspect_err` once stable
        .map_err(|err| {
            tracing::error!(exception.stacktrace = ?err, message= %err);
            err
        })
        .instrument(span)
        .await
}

async fn execute_with_metrics(
    req: Request,
    client: &reqwest::Client,
) -> reqwest::Result<reqwest::Response> {
    let start = Instant::now();
    let method = req.method().as_str().to_string();
    let url = req.url();
    let path = url.path().to_string();
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
            );
            increment_counter!(
                "http_client_requests_count",
                "method" => method,
                "status_code" => status_code,
                "path" => path,
                "host" => host,
            );
            res
        })
        .and_then(|r| r.error_for_status())
}
