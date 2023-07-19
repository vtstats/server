use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    fmt::format::FmtSpan,
    layer::SubscriberExt,
    registry, Layer,
};

pub fn init() -> WorkerGuard {
    let filter_layer = filter_fn(|metadata| {
        metadata.target().starts_with("vtstat") && metadata.name() != "Ignored"
    });

    let log_dir = std::env::var("LOG_DIR").unwrap_or("/var/log/vtstat".into());

    let file_appender = tracing_appender::rolling::daily(log_dir, "log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    let subscriber = registry().with(filter_layer).with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to initialize tracing subscriber");

    guard
}
