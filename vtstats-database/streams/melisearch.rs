use crate::{
    channels::Platform,
    streams::{Stream, StreamStatus},
};
use chrono::{DateTime, Utc};
use meilisearch_sdk::{
    documents::{DocumentQuery, DocumentsQuery, DocumentsResults},
    errors::Error,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    cmp::{max, min},
    fmt::Display,
    fmt::Write,
};

// re-export client
pub use meilisearch_sdk::client::Client;

use super::{Column, Ordering};

#[skip_serializing_none]
#[derive(Default, Serialize)]
pub struct Document<'q> {
    pub stream_id: i32,
    pub vtuber_id: Option<&'q str>,
    pub platform: Option<Platform>,
    pub platform_stream_id: Option<&'q str>,
    pub channel_id: Option<i32>,
    pub title: Option<&'q str>,
    pub status: Option<StreamStatus>,
    pub thumbnail_url: Option<String>,
    pub schedule_time: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub like_max: Option<i32>,
}

pub async fn add_or_update<'a>(patch: Document<'a>, client: &Client) -> Result<(), Error> {
    let index = client.index("streams");

    let _task = index
        .add_or_update(&[&patch], Some("stream_id".into()))
        .await?;

    Ok(())
}

pub async fn find_by_platform_id(
    platform_id: String,
    platform: Platform,
    client: &Client,
) -> Result<Option<Stream>, Error> {
    let index = client.index("streams");

    let mut documents: DocumentsResults<_> = DocumentsQuery::new(&index)
        .with_filter(&format!(
            "platform_id = '{platform_id}' AND platform = '{}'",
            match platform {
                Platform::Youtube => "youtube",
                Platform::Bilibili => "bilibili",
                Platform::Twitch => "twitch",
            }
        ))
        .with_limit(1)
        .execute::<Stream>()
        .await?;

    Ok(documents.results.pop())
}

pub async fn find_by_id(stream_id: i32, client: &Client) -> Result<Option<Stream>, Error> {
    let index = client.index("streams");

    let query = DocumentQuery::new(&index)
        .execute::<Stream>(&stream_id.to_string())
        .await?;

    Ok(Some(query))
}

pub async fn delete(stream_id: i32, client: &Client) -> Result<(), Error> {
    let index = client.index("streams");

    index.delete_document(stream_id).await?;

    Ok(())
}

#[derive(Default)]
pub struct Search {
    pub channel_ids: Vec<i32>,
    pub status: StreamStatus,
    pub query: Option<String>,
    pub sort_by: Column,
    pub sort_direction: Ordering,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
}

pub async fn search(search: Search, client: &Client) -> Result<Vec<Stream>, Error> {
    if search.channel_ids.is_empty() {
        return Ok(vec![]);
    }

    let index = client.index("streams");

    let mut filter = format!(
        "status = '{}' AND {}",
        match search.status {
            StreamStatus::Scheduled => "scheduled",
            StreamStatus::Live => "live",
            StreamStatus::Ended => "ended",
        },
        ArrayFilter("channel_id", &search.channel_ids)
    );

    if let Some(start_at) = search.start_at {
        let _ = write!(
            &mut filter,
            " AND {} > '{start_at}'",
            search.sort_by.as_str()
        );
    };

    if let Some(end_at) = search.end_at {
        let _ = write!(&mut filter, " AND {} < '{end_at}'", search.sort_by.as_str());
    };

    let result = index
        .search()
        .with_query(&search.query.unwrap_or_default())
        .with_filter(&filter)
        .with_sort(&[&format!(
            "{}:{}",
            search.sort_by.as_str(),
            search.sort_direction.as_str().to_lowercase()
        )])
        .with_limit(24)
        .execute::<Stream>()
        .await?;

    Ok(result.hits.into_iter().map(|x| x.result).collect())
}

pub async fn stream_times(
    channel_ids: &[i32],
    start_at: DateTime<Utc>,
    client: &Client,
) -> Result<Vec<(i64, i64)>, Error> {
    if channel_ids.is_empty() {
        return Ok(vec![]);
    }

    #[derive(Deserialize)]
    struct Result {
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    }

    let index = client.index("streams");

    let result: DocumentsResults<_> = DocumentsQuery::new(&index)
        .with_filter(&format!(
            "{} AND start_time > {start_at} AND end_time IS NOT NULL",
            ArrayFilter("channel_id", channel_ids)
        ))
        .with_limit(usize::MAX)
        .with_fields(["start_time", "end_time"])
        .execute::<Result>()
        .await?;

    let mut streams = result.results;

    streams.sort_by(|a, b| a.start_time.cmp(&b.start_time));

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

struct ArrayFilter<'a>(&'a str, &'a [i32]);

impl<'a> Display for ArrayFilter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1.is_empty() {
            return Ok(());
        };

        f.write_str(&self.0)?;

        if self.1.len() == 1 {
            f.write_str(" = ")?;
            return f.write_str(&self.1[0].to_string());
        }

        f.write_str(" IN [")?;
        for (index, id) in self.1.iter().enumerate() {
            if index != 0 {
                f.write_str(",")?;
            }
            f.write_str(&id.to_string())?;
        }
        f.write_str("]")
    }
}
