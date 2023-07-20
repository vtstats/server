use std::{env, net::SocketAddr, time::Duration};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;

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
