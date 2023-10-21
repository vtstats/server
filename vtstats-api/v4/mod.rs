mod catalog;
mod channel_stats;
mod channels;
mod exchange_rates;
mod stream_events;
mod stream_stats;
mod stream_times;
mod streams;

pub use catalog::*;
pub use channel_stats::*;
pub use channels::*;
pub use exchange_rates::*;
pub use stream_events::*;
pub use stream_stats::*;
pub use stream_times::*;
pub use streams::*;

use axum::routing::get;
use axum::Router;
use vtstats_database::PgPool;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/catalog", get(catalog))
        .route("/exchange-rates", get(exchange_rates))
        .route("/channel-stats/summary", get(channel_stats_summary))
        .route("/stream-stats/viewer", get(stream_viewer_stats))
        .route("/stream-stats/chat", get(stream_chat_stats))
        .route("/stream-events", get(stream_events))
        .route("/stream-times", get(stream_times))
        .route("/channel-stats/subscriber", get(channel_subscriber_stats))
        .route("/channel-stats/view", get(channel_view_stats))
        .route("/channel-stats/revenue", get(channel_revenue_stats))
        .route("/streams", get(find_stream_by_id))
        .route("/streams/scheduled", get(list_scheduled_streams))
        .route("/streams/live", get(list_live_streams))
        .route("/streams/ended", get(list_ended_streams))
        .with_state(pool)
}
