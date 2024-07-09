use crate::{
    channels::Platform,
    streams::{Stream, StreamStatus},
};
use chrono::{serde::ts_milliseconds, serde::ts_milliseconds_option, DateTime, Utc};
use meilisearch_sdk::{
    documents::{DocumentQuery, DocumentsQuery, DocumentsResults},
    errors::Error,
};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{fmt::Display, fmt::Write};

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
    #[serde(with = "ts_milliseconds_option")]
    pub schedule_time: Option<DateTime<Utc>>,
    #[serde(with = "ts_milliseconds_option")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(with = "ts_milliseconds_option")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(with = "ts_milliseconds")]
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
        ArrayFieldFilter("channel_id", &search.channel_ids)
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

pub struct ArrayFieldFilter<'a>(pub &'a str, pub &'a [i32]);

impl<'a> Display for ArrayFieldFilter<'a> {
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
