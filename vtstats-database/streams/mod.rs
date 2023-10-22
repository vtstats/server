mod delete_stream;
mod end_stream;
mod find_stream;
mod get_stream_by_id;
mod get_stream_by_platform_id;
mod list_streams;
mod start_stream;
mod stream_times;
mod update_stream_title;
mod upsert_stream;

pub use self::delete_stream::*;
pub use self::end_stream::*;
pub use self::find_stream::*;
pub use self::get_stream_by_id::*;
pub use self::get_stream_by_platform_id::*;
pub use self::list_streams::*;
pub use self::start_stream::*;
pub use self::stream_times::*;
pub use self::update_stream_title::*;
pub use self::upsert_stream::*;

use crate::channels::Platform;
use sqlx::{PgPool, Result};

pub struct Record {
    pub platform: Platform,
    pub platform_id: String,
}

pub async fn list_stream_ids(pool: &PgPool) -> Result<Vec<Record>> {
    let query = sqlx::query_as!(
        Record,
        "SELECT platform \"platform: _\", platform_id FROM streams"
    )
    .fetch_all(pool);
    crate::otel::execute_query!("SELECT", "vtubers", query)
}
