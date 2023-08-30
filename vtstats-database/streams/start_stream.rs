use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool, Result};

pub async fn start_stream(
    stream_id: i32,
    title: Option<&str>,
    start_time: DateTime<Utc>,
    likes: Option<i32>,
    pool: &PgPool,
) -> Result<PgQueryResult> {
    let query = sqlx::query!(
        r#"
 UPDATE streams
    SET title      = COALESCE($1, title),
        updated_at = NOW(),
        start_time = COALESCE(start_time, $2),
        status     = 'live',
        like_max   = GREATEST($3, like_max)
  WHERE stream_id  = $4
        "#,
        title,      // $1
        start_time, // $2
        likes,      // $3
        stream_id,  // $4
    )
    .execute(pool);

    crate::otel::execute_query!("UPDATE", "streams", query)
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query!(
        r#"
INSERT INTO streams (stream_id, vtuber_id, title, channel_id, platform_id, platform, schedule_time, start_time, end_time, status)
     VALUES (1, 'vtuber1', 'title1', 1, 'id1', 'youtube', to_timestamp(0), NULL, NULL, 'scheduled'),
            (2, 'vtuber1', 'title2', 2, 'id2', 'youtube', to_timestamp(0), to_timestamp(10000), to_timestamp(12000), 'live'),
            (3, 'vtuber1', 'title3', 2, 'id3', 'youtube', to_timestamp(10000), to_timestamp(15000), to_timestamp(17000), 'ended');
        "#
    )
    .execute(&pool)
    .await?;

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        let res = start_stream(1, None, time, None, &pool).await?;

        let row = sqlx::query!("SELECT status::TEXT, start_time FROM streams WHERE stream_id = 1")
            .fetch_one(&pool)
            .await?;

        assert_eq!(res.rows_affected(), 1);
        assert_eq!(row.status, Some("live".into()));
        assert_eq!(row.start_time, Some(time));
    }

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        let res = start_stream(2, Some("title_alt"), time, Some(100), &pool).await?;

        let row = sqlx::query!(
            "SELECT status::TEXT, start_time, title, like_max FROM streams WHERE stream_id = 2"
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(res.rows_affected(), 1);
        assert_eq!(row.status, Some("live".into()));
        assert_eq!(row.title, "title_alt".to_string());
        assert_eq!(
            row.start_time,
            Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(10000, 0).unwrap(),
                Utc
            ))
        );
        assert_eq!(row.like_max, Some(100));
    }

    Ok(())
}
