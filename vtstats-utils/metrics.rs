use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use std::{env, net::SocketAddr, time::Duration};

pub use tracing;

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
