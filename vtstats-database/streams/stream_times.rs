use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Result};
use std::cmp::{max, min};

pub async fn stream_times(channel_ids: &[i32], pool: &PgPool) -> Result<Vec<(i64, i64)>> {
    stream_times_start_at(channel_ids, Utc::now() - Duration::weeks(44), pool).await
}

async fn stream_times_start_at(
    channel_ids: &[i32],
    start_at: DateTime<Utc>,
    pool: &PgPool,
) -> Result<Vec<(i64, i64)>> {
    let query = sqlx::query!(
        r#"
  SELECT start_time, end_time
    FROM streams
   WHERE channel_id = ANY($1)
     AND start_time > $2
     AND end_time IS NOT NULL
ORDER BY start_time DESC
        "#,
        channel_ids, // $1
        start_at,    // $2
    )
    .fetch_all(pool);

    let records = crate::otel::execute_query!("SELECT", "streams", query)?;

    let mut result = Vec::<(i64, i64)>::new();

    for record in records {
        let (Some(start), Some(end)) = (record.start_time, record.end_time) else {
            continue;
        };

        let start = start.timestamp();
        let end = end.timestamp();
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

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::TimeZone;

    sqlx::query!(
        r#"
INSERT INTO streams (platform, vtuber_id, platform_id, title, channel_id, schedule_time, start_time, end_time, status)
     VALUES ('youtube', 'vtuber1', 'id1', 'title1', 1, NULL, to_timestamp(1800), to_timestamp(8000), 'ended'),
            ('youtube', 'vtuber1', 'id2', 'title2', 1, NULL, to_timestamp(10000), to_timestamp(12000), 'ended'),
            ('youtube', 'vtuber1', 'id3', 'title3', 1, NULL, to_timestamp(15000), to_timestamp(17000), 'ended');
        "#
    )
    .execute(&pool)
    .await?;

    let big_bang = Utc.timestamp_opt(0, 0).single().unwrap();

    {
        let times = stream_times_start_at(&[2], big_bang, &pool).await?;

        assert!(times.is_empty());
    }

    {
        let times = stream_times_start_at(&[1], big_bang, &pool).await?;

        assert_eq!(
            times,
            vec![
                (14400, 2000),
                (10800, 1200),
                (7200, 1600),
                (3600, 3600),
                (0, 1800),
            ]
        );
    }

    Ok(())
}
