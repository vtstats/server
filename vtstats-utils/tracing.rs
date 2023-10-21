use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{self, Write},
    time::Instant,
};
use tracing::{
    field::{Field, Visit},
    span::{Attributes, Record},
    Event, Id, Metadata, Subscriber,
};
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    layer::{Context, SubscriberExt},
    registry,
    registry::LookupSpan,
    Layer,
};

pub fn init() {
    let filter_layer = filter_fn(|metadata| {
        (metadata.target().starts_with("vtstats") || metadata.target().starts_with("integration"))
            && metadata.name() != "Ignored"
    });

    let subscriber = registry()
        .with(LevelFilter::INFO)
        .with(filter_layer)
        .with(JsonMessageLayer::new());

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to initialize tracing subscriber");
}

#[derive(serde::Serialize, Debug)]
struct JsonMessage {
    target: &'static str,
    level: &'static str,
    message: String,

    #[serde(flatten)]
    fields: BTreeMap<&'static str, Value>,

    #[serde(skip)]
    file: Option<&'static str>,
    #[serde(skip)]
    line: Option<u32>,
    #[serde(skip)]
    start: Instant,
}

impl JsonMessage {
    fn new(metadata: &Metadata<'static>) -> JsonMessage {
        JsonMessage {
            start: Instant::now(),
            file: metadata.file(),
            line: metadata.line(),
            target: metadata.target(),
            message: metadata.name().into(),
            level: metadata.level().as_str(),
            fields: BTreeMap::new(),
        }
    }

    fn print(mut self, mut w: &io::Stdout) {
        if let Some(Value::String(message)) = self.fields.remove("message") {
            self.message = message;
        }

        if let (Some(file), Some(line)) = (self.file, self.line) {
            self.message += &format!(" caller={file}:{line}");
        }

        let ms = self.start.elapsed().as_millis();
        if ms > 0 {
            self.message += &format!(" duration={ms}ms");
        }

        if let Some(job_id) = self.fields.remove("job_id") {
            self.message += &format!(" job_id={job_id}");
        }

        if let Some(stream_id) = self.fields.remove("stream_id") {
            self.message += &format!(" stream_id={stream_id}");
        }

        if std::env::var("LOG_PRETTY")
            .map(|c| c == "1" || c == "on" || c == "true")
            .unwrap_or_default()
        {
            let _ = serde_json::to_writer_pretty(w, &self);
        } else {
            let _ = serde_json::to_writer(w, &self);
        }

        let _ = w.write_all(b"\n");
    }
}

pub struct JsonMessageLayer {
    stdout: io::Stdout,
}

impl JsonMessageLayer {
    pub fn new() -> Self {
        JsonMessageLayer {
            stdout: io::stdout(),
        }
    }
}

impl<S> Layer<S> for JsonMessageLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("span not found");

        let mut msg = JsonMessage::new(span.metadata());

        attrs.record(&mut msg);

        span.extensions_mut().insert(msg);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut msg = JsonMessage::new(event.metadata());
        event.record(&mut msg);

        if let Some(id) = ctx.current_span().id() {
            let span = ctx.span(id).expect("span not found");
            let extensions = span.extensions();

            if let Some(span_msg) = extensions.get::<JsonMessage>() {
                msg.fields.extend(
                    span_msg
                        .fields
                        .iter()
                        // inherit fields from span expect "message" field
                        .filter_map(|(k, v)| (*k != "message").then(|| (*k, v.clone()))),
                );
            }
        }

        msg.print(&self.stdout);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("span not found");
        let mut extensions = span.extensions_mut();
        if let Some(msg) = extensions.get_mut::<JsonMessage>() {
            values.record(msg);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("span not found");
        let mut extensions = span.extensions_mut();
        if let Some(msg) = extensions.remove::<JsonMessage>() {
            msg.print(&self.stdout);
        }
    }
}

impl Visit for JsonMessage {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name(), Value::from(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name(), Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name(), Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name(), Value::from(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name(), Value::from(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_str(field, &format!("{}", value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_str(field, &format!("{:?}", value));
    }
}
