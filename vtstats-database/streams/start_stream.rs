use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use sqlx::{PgPool, Result};

use super::StreamStatus;

pub async fn start_stream(
    stream_id: i32,
    title: Option<&str>,
    start_time: DateTime<Utc>,
    likes: Option<i32>,
    pool: &PgPool,
    client: &Client,
) -> Result<()> {
    let now = Utc::now();

    let query = sqlx::query!(
        "UPDATE streams \
        SET title = COALESCE($2, title), \
        updated_at = $1, \
        start_time = $3, \
        status = 'live', \
        like_max = GREATEST($4, like_max) \
        WHERE stream_id = $5 \
        RETURNING like_max",
        now,        // $1
        title,      // $2
        start_time, // $3
        likes,      // $4
        stream_id,  // $5
    )
    .fetch_optional(pool);

    let rec = crate::otel::execute_query!("UPDATE", "streams", query)?;

    if let Some(rec) = rec {
        if let Err(err) = super::melisearch::add_or_update(
            super::melisearch::Document {
                stream_id,
                title,
                updated_at: now,
                start_time: Some(start_time),
                status: Some(StreamStatus::Live),
                like_max: rec.like_max,
                ..Default::default()
            },
            client,
        )
        .await
        {
            eprintln!("meili: {err:?}");
        }
    }

    Ok(())
}

// #[cfg(test)]
// #[sqlx::test(fixtures("channels"))]
// async fn test(pool: PgPool) -> Result<()> {
//     use chrono::TimeZone;

//     sqlx::query!(
//         r#"
// INSERT INTO streams (stream_id, vtuber_id, title, channel_id, platform_id, platform, schedule_time, start_time, end_time, status)
//      VALUES (1, 'vtuber1', 'title1', 1, 'id1', 'youtube', to_timestamp(0), NULL, NULL, 'scheduled'),
//             (2, 'vtuber1', 'title2', 2, 'id2', 'youtube', to_timestamp(0), to_timestamp(10000), to_timestamp(12000), 'live'),
//             (3, 'vtuber1', 'title3', 2, 'id3', 'youtube', to_timestamp(10000), to_timestamp(15000), to_timestamp(17000), 'ended');
//         "#
//     )
//     .execute(&pool)
//     .await?;

//     {
//         let time = Utc.timestamp_opt(3000, 0).single().unwrap();

//         let res = start_stream(1, None, time, None, &pool).await?;

//         let row = sqlx::query!("SELECT status::TEXT, start_time FROM streams WHERE stream_id = 1")
//             .fetch_one(&pool)
//             .await?;

//         assert_eq!(res.rows_affected(), 1);
//         assert_eq!(row.status, Some("live".into()));
//         assert_eq!(row.start_time, Some(time));
//     }

//     {
//         let time = Utc.timestamp_opt(3000, 0).single().unwrap();

//         let res = start_stream(2, Some("title_alt"), time, Some(100), &pool).await?;

//         let row = sqlx::query!(
//             "SELECT status::TEXT, start_time, title, like_max FROM streams WHERE stream_id = 2"
//         )
//         .fetch_one(&pool)
//         .await?;

//         assert_eq!(res.rows_affected(), 1);
//         assert_eq!(row.status, Some("live".into()));
//         assert_eq!(row.title, "title_alt".to_string());
//         assert_eq!(
//             row.start_time,
//             Some(Utc.timestamp_opt(10000, 0).single().unwrap())
//         );
//         assert_eq!(row.like_max, Some(100));
//     }

//     Ok(())
// }
