mod catalog;
mod channel_stats;
mod channels;
mod exchange_rates;
mod stream_events;
mod stream_stats;
mod stream_times;
mod streams;

use catalog::*;
use channel_stats::*;
use channels::*;
use exchange_rates::*;
use stream_events::*;
use stream_stats::*;
use stream_times::*;
use streams::*;

use vtstats_database::PgPool;
use warp::{Filter, Rejection, Reply};

use crate::filters::with_pool;

pub fn routes(pool: PgPool) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_channel_stats_summary = warp::path!("channel-stats" / "summary")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_stats_summary);

    let api_channel_stats_subscriber = warp::path!("channel-stats" / "subscriber")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_subscriber_stats);

    let api_channel_stats_view = warp::path!("channel-stats" / "view")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_view_stats);

    let api_channel_stats_revenue = warp::path!("channel-stats" / "revenue")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_revenue_stats);

    let api_get_stream = warp::path!("streams")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(find_stream_by_id);

    let api_scheduled_streams = warp::path!("streams" / "scheduled")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(list_scheduled_streams);

    let api_live_streams = warp::path!("streams" / "live")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(list_live_streams);

    let api_ended_streams = warp::path!("streams" / "ended")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(list_ended_streams);

    let api_stream_stats_viewer = warp::path!("stream-stats" / "viewer")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_viewer_stats);

    let api_stream_stats_chat = warp::path!("stream-stats" / "chat")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_chat_stats);

    let api_stream_times = warp::path!("stream-times")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_times);

    let api_stream_stats = warp::path!("stream-events")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_events);

    let api_exchange_rates = warp::path!("exchange-rates")
        .and(with_pool(pool.clone()))
        .and_then(exchange_rates);

    let api_catalog = warp::path!("catalog")
        .and(with_pool(pool.clone()))
        .and_then(catalog);

    warp::path("v4").and(warp::get()).and(
        api_channel_stats_summary
            .or(api_catalog)
            .or(api_channel_stats_subscriber)
            .or(api_channel_stats_view)
            .or(api_channel_stats_revenue)
            .or(api_get_stream)
            .or(api_scheduled_streams)
            .or(api_live_streams)
            .or(api_ended_streams)
            .or(api_stream_stats_viewer)
            .or(api_stream_stats_chat)
            .or(api_stream_times)
            .or(api_stream_stats)
            .or(api_exchange_rates),
    )
}
