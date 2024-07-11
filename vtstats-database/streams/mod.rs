mod delete_stream;
mod end_stream;
mod get_stream_by_id;
mod get_stream_by_platform_id;
mod if_stream_is_live;
mod list_streams;
pub mod meilisearch;
mod start_stream;
mod stream_times;
mod upsert_stream;

pub use self::delete_stream::*;
pub use self::end_stream::*;
pub use self::get_stream_by_id::*;
pub use self::get_stream_by_platform_id::*;
pub use self::if_stream_is_live::*;
pub use self::list_streams::*;
pub use self::start_stream::*;
pub use self::stream_times::*;
pub use self::upsert_stream::*;

use crate::channels::Platform;
use meilisearch_sdk::{
    client::Client,
    documents::{DocumentsQuery, DocumentsResults},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Document {
    pub platform: Platform,
    pub platform_id: String,
}

pub async fn list_stream_ids(client: &Client) -> anyhow::Result<Vec<Document>> {
    let index = client.index("streams");

    let result: DocumentsResults<_> = DocumentsQuery::new(&index)
        .with_limit(1_000_000)
        .with_fields(["platform", "platformId"])
        .execute::<Document>()
        .await?;

    Ok(result.results)
}
