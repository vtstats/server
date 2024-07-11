use crate::{
    channels::Platform,
    streams::{Stream, StreamStatus},
};
use chrono::{serde::ts_milliseconds, serde::ts_milliseconds_option, DateTime, Utc};
use meilisearch_sdk::{client::Client, errors::Error};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{fmt::Display, fmt::Write};

use super::{Column, Ordering};

#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document<'q> {
    pub stream_id: i32,
    pub vtuber_id: Option<&'q str>,
    pub platform: Option<Platform>,
    pub platform_id: Option<&'q str>,
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
    #[serde(default)]
    pub viewer_avg: Option<i32>,
    #[serde(default)]
    pub viewer_max: Option<i32>,
}

pub async fn add_or_update<'a>(patch: Document<'a>, client: &Client) -> Result<(), Error> {
    let index = client.index("streams");

    let _task = index
        .add_or_update(&[&patch], Some("streamId".into()))
        .await?;

    Ok(())
}

pub async fn delete(stream_id: i32, client: &Client) -> Result<(), Error> {
    let index = client.index("streams");

    index.delete_document(stream_id).await?;

    Ok(())
}

pub struct Search {
    pub channel_ids: Option<Vec<i32>>,
    pub status: StreamStatus,
    pub query: Option<String>,
    pub sort_by: Column,
    pub sort_direction: Ordering,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
}

pub async fn search(search: Search, client: &Client) -> Result<Vec<Stream>, Error> {
    if matches!(&search.channel_ids, Some(ids) if ids.is_empty()) {
        return Ok(vec![]);
    }

    let index = client.index("streams");

    let mut filter = format!("status = {}", search.status.lowercase());

    if let Some(channel_ids) = search.channel_ids {
        let _ = write!(
            &mut filter,
            " AND {}",
            ArrayFieldFilter("channelId", &channel_ids)
        );
    }

    if let Some(start_at) = search.start_at {
        let _ = write!(
            &mut filter,
            " AND {} > {}",
            search.sort_by.camelcase(),
            start_at.timestamp_millis()
        );
    };

    if let Some(end_at) = search.end_at {
        let _ = write!(
            &mut filter,
            " AND {} < {}",
            search.sort_by.camelcase(),
            end_at.timestamp_millis()
        );
    };

    let search_query = &mut index.search();

    if let Some(query) = search
        .query
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        search_query.query = Some(&query);
    }

    let result = search_query
        .with_filter(&filter)
        .with_sort(&[&format!(
            "{}:{}",
            search.sort_by.camelcase(),
            search.sort_direction.lowercase()
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
