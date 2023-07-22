use metrics::{histogram, increment_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use std::{
    env,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tracing::{
    field::{debug, display, Empty},
    Instrument, Span,
};

pub fn install() {
    let mut builder = PrometheusBuilder::new().idle_timeout(
        MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
        Some(Duration::from_secs(10)),
    );

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

    let span = tracing::info_span!(
        "HTTP Client",
        //// request
        http.method = &method,
        http.url = url.as_str(),
        http.path = &path,
        http.request_content_length = request_content_length,
        http.requests_elapsed_seconds = Empty,
        ///// response
        http.status_code = Empty,
        http.response_content_length = Empty,
        //// error
        error.message = Empty,
        error.cause_chain = Empty,
    );

    async move {
        let start = Instant::now();

        let res = client.execute(req).await?;

        Span::current()
            .record("http.status_code", res.status().as_u16())
            .record("http.response_content_length", res.content_length());

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

        let result = res.error_for_status();

        // TODO: use `inspect_err` once stable
        if let Err(err) = &result {
            Span::current()
                .record("error.message", display(err))
                .record("error.cause_chain", debug(err));
        }

        result
    }
    .instrument(span)
    .await
}
