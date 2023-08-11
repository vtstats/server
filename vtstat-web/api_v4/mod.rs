mod catalog;
mod channel_stats;
mod channels;
mod currencies;
mod stream_events;
mod stream_stats;
mod stream_times;
mod streams;

use catalog::*;
use channel_stats::*;
use channels::*;
use currencies::*;
use stream_events::*;
use stream_stats::*;
use stream_times::*;
use streams::*;

use vtstat_database::PgPool;
use warp::{Filter, Rejection, Reply};

use crate::filters::with_pool;

pub fn routes(pool: PgPool) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_channels = warp::path!("channels")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(list_channels);

    let api_channel_subscriber_stats = warp::path!("channel_stats" / "subscriber")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_subscriber_stats);

    let api_channel_view_stats = warp::path!("channel_stats" / "view")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_view_stats);

    let api_channel_revenue_stats = warp::path!("channel_stats" / "revenue")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(channel_revenue_stats);

    let api_currencies = warp::path!("currencies")
        .and(with_pool(pool.clone()))
        .and_then(list_currencies);

    let api_get_stream = warp::path!("streams")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(list_stream_by_platform_id);

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

    let api_stream_viewer_stats = warp::path!("stream_stats" / "viewer")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_viewer_stats);

    let api_stream_chat_stats = warp::path!("stream_stats" / "chat")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_chat_stats);

    let api_stream_times = warp::path!("stream_times")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_times);

    let api_stream_stats = warp::path!("stream_events")
        .and(warp::query())
        .and(with_pool(pool.clone()))
        .and_then(stream_events);

    let api_catalog = warp::path!("catalog")
        .and(with_pool(pool.clone()))
        .and_then(catalog);

    warp::path("v4").and(warp::get()).and(
        api_channels
            .or(api_catalog)
            .or(api_channel_subscriber_stats)
            .or(api_channel_view_stats)
            .or(api_channel_revenue_stats)
            .or(api_currencies)
            .or(api_get_stream)
            .or(api_scheduled_streams)
            .or(api_live_streams)
            .or(api_ended_streams)
            .or(api_stream_viewer_stats)
            .or(api_stream_chat_stats)
            .or(api_stream_times)
            .or(api_stream_stats),
    )
}
