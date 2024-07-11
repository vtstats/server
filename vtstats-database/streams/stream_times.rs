use chrono::{serde::ts_milliseconds, DateTime, Duration, Utc};
use meilisearch_sdk::{
    client::Client,
    documents::{DocumentsQuery, DocumentsResults},
};
use serde::Deserialize;
use std::cmp::{max, min};

use super::meilisearch::ArrayFieldFilter;

pub async fn stream_times(channel_ids: &[i32], client: &Client) -> anyhow::Result<Vec<(i64, i64)>> {
    stream_times_start_at(channel_ids, Utc::now() - Duration::weeks(44), client).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Document {
    #[serde(with = "ts_milliseconds")]
    start_time: DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    end_time: DateTime<Utc>,
}

async fn stream_times_start_at(
    channel_ids: &[i32],
    start_at: DateTime<Utc>,
    client: &Client,
) -> anyhow::Result<Vec<(i64, i64)>> {
    if channel_ids.is_empty() {
        return Ok(vec![]);
    }

    let index = client.index("streams");

    let filter = format!(
        "{} AND startTime > {} AND endTime EXISTS",
        ArrayFieldFilter("channelId", channel_ids),
        start_at.timestamp_millis()
    );

    let result: DocumentsResults<_> = DocumentsQuery::new(&index)
        .with_filter(&filter)
        .with_limit(1_000_000)
        .with_fields(["startTime", "endTime"])
        .execute::<Document>()
        .await?;

    let mut streams = result.results;

    streams.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    let mut result = Vec::<(i64, i64)>::new();

    for stream in streams {
        let start = stream.start_time.timestamp();
        let end = stream.end_time.timestamp();
        let one_hour: i64 = 60 * 60;

        let mut time = end - (end % one_hour);

        while (start - time) < one_hour {
            let duration = min(time + one_hour, end) - max(start, time);

            match result.last_mut() {
                Some(last) if last.0 == time => {
                    last.1 += duration;
                }
                _ => result.push((time, duration)),
            }

            time -= one_hour;
        }
    }

    Ok(result)
}
