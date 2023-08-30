use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub async fn end_stream(stream_id: i32, pool: &PgPool) -> Result<()> {
    let query = sqlx::query!(
        "UPDATE streams SET status = 'ended', end_time = NOW() WHERE stream_id = $1",
        stream_id
    )
    .execute(pool);

    crate::otel::execute_query!("UPDATE", "streams", query)?;

    Ok(())
}

pub async fn end_stream_with_values(
    stream_id: i32,
    title: Option<&str>,
    schedule_time: Option<DateTime<Utc>>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    likes: Option<i32>,
    pool: &PgPool,
) -> Result<()> {
    let query = sqlx::query!(
        r#"
 UPDATE streams
    SET status        = 'ended',
        title         = COALESCE($1, title),
        end_time      = COALESCE($2, end_time),
        start_time    = COALESCE($3, start_time),
        schedule_time = COALESCE($4, schedule_time),
        like_max      = GREATEST($5, like_max),
        updated_at    = NOW()
  WHERE stream_id     = $6
        "#,
        title,         // $1
        end_time,      // $2
        start_time,    // $3
        schedule_time, // $4
        likes,         // $5
        stream_id,     // $6
    )
    .execute(pool);

    crate::otel::execute_query!("UPDATE", "streams", query)?;

    Ok(())
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query!(
        r#"
    INSERT INTO streams (stream_id, vtuber_id, platform, platform_id, title, like_max, status, channel_id)
         VALUES (1, 'vtuber1', 'youtube', 'id1', 'title1', 100, 'live', 1),
                (2, 'vtuber1', 'youtube', 'id2', 'title2', 100, 'scheduled', 1),
                (3, 'vtuber1', 'youtube', 'id3', 'title3', 100, 'ended', 1);
    "#
    )
    .execute(&pool)
    .await?;

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(9000, 0).unwrap(), Utc);

        end_stream_with_values(1, None, None, None, Some(time), Some(0), &pool).await?;

        let row = sqlx::query!(
            r#"SELECT status::TEXT, like_max, end_time FROM streams WHERE stream_id = 1"#
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(row.status, Some("ended".into()));
        assert_eq!(row.like_max, Some(100));
        assert_eq!(row.end_time, Some(time));
    }

    {
        end_stream_with_values(3, None, None, None, None, Some(200), &pool).await?;

        let row = sqlx::query!(
            r#"SELECT status::TEXT, like_max, end_time FROM streams WHERE stream_id = 3"#
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(row.status, Some("ended".into()));
        assert_eq!(row.like_max, Some(200));
        assert_eq!(row.end_time, None);
    }

    Ok(())
}
