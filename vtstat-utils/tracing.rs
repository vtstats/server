use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    fmt::format::FmtSpan,
    layer::SubscriberExt,
    registry,
};

pub fn init() {
    let filter_layer = filter_fn(|metadata| {
        metadata.target().starts_with("vtstat") && metadata.name() != "Ignored"
    });

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_level(true)
        .flatten_event(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true)
        .with_span_list(false);

    let subscriber = registry()
        .with(LevelFilter::INFO)
        .with(filter_layer)
        .with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to initialize tracing subscriber");
}
