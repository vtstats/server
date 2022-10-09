use log::Log;
use std::env;
use tracing::{Metadata, Subscriber};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::format::FmtSpan,
    layer::{Context, SubscriberExt},
    registry,
    registry::LookupSpan,
    Layer,
};

// logger
struct TelemetryLogger;

impl Log for TelemetryLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().starts_with("tracing_newrelic")
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("[NewRelic] {}", record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: TelemetryLogger = TelemetryLogger;

// target filter
struct TargetFilterLayer(&'static str);

impl<S> Layer<S> for TargetFilterLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
        let target = metadata.target();
        target.starts_with(self.0) || target.starts_with("vtstat")
    }
}

pub fn init(target: &'static str) {
    // initialize logger
    log::set_max_level(log::LevelFilter::Info);
    log::set_logger(&LOGGER).expect("failed to initialize telemetry logger");

    let newrelic_layer = match env::var("NEWRELIC_API_KEY") {
        Ok(api_key) => Some(tracing_newrelic::layer(api_key.as_str())),
        Err(_) => {
            println!("`NEWRELIC_API_KEY` not found");
            None
        }
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    let subscriber = registry()
        .with(fmt_layer)
        .with(newrelic_layer)
        .with(TargetFilterLayer(target));

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to initialize tracing subscriber");
}
