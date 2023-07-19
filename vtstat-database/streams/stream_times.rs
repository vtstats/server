use chrono::{DateTime, Duration, Utc};
use futures_util::stream::TryStreamExt;
use sqlx::{postgres::PgRow, PgPool, Result, Row};
use std::cmp::{max, min};

type UtcTime = DateTime<Utc>;

pub struct StreamTimesQuery<'q> {
    pub vtuber_id: &'q str,
    pub start_at: UtcTime,
}

impl<'q> Default for StreamTimesQuery<'q> {
    fn default() -> Self {
        StreamTimesQuery {
            vtuber_id: "",
            start_at: Utc::now() - Duration::weeks(44),
        }
    }
}

impl<'q> StreamTimesQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<(i64, i64)>> {
        let query = sqlx::query(
            r#"
     SELECT start_time, end_time
       FROM streams
      WHERE channel_id IN
            (
                SELECT channel_id FROM channels WHERE vtuber_id = $1
            )
        AND start_time > $2
        AND end_time IS NOT NULL
   ORDER BY start_time DESC
            "#,
        )
        .bind(self.vtuber_id) // $1
        .bind(self.start_at) // $2
        .fetch(pool)
        .try_fold(Vec::<(i64, i64)>::new(), |mut acc, row: PgRow| async move {
            let start = row.try_get::<DateTime<Utc>, _>("start_time")?.timestamp();
            let end = row.try_get::<DateTime<Utc>, _>("end_time")?.timestamp();
            let one_hour: i64 = 60 * 60;

            let mut time = end - (end % one_hour);

            while (start - time) < one_hour {
                let duration = min(time + one_hour, end) - max(start, time);

                match acc.last_mut() {
                    Some(last) if last.0 == time => {
                        last.1 += duration;
                    }
                    _ => acc.push((time, duration)),
                }

                time -= one_hour;
            }

            Ok(acc)
        });

        crate::otel::instrument("SELECT", "streams", query).await
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query!(
        r#"
INSERT INTO streams (platform, platform_id, title, channel_id, schedule_time, start_time, end_time, status)
     VALUES ('youtube', 'id1', 'title1', 1, NULL, to_timestamp(1800), to_timestamp(8000), 'ended'),
            ('youtube', 'id2', 'title2', 1, NULL, to_timestamp(10000), to_timestamp(12000), 'ended'),
            ('youtube', 'id3', 'title3', 1, NULL, to_timestamp(15000), to_timestamp(17000), 'ended');      
        "#
    )
    .execute(&pool)
    .await?;

    let big_bang = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc);

    {
        let times = StreamTimesQuery {
            vtuber_id: "vtuber2",
            start_at: big_bang,
        }
        .execute(&pool)
        .await?;

        assert!(times.is_empty());
    }

    {
        let times = StreamTimesQuery {
            vtuber_id: "vtuber1",
            start_at: big_bang,
        }
        .execute(&pool)
        .await?;

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
