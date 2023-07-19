use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, PgPool, Result, Row};

type UtcTime = DateTime<Utc>;

pub struct YouTubeStreamsLastUpdated;

impl YouTubeStreamsLastUpdated {
    pub async fn execute(self, pool: &PgPool) -> Result<Option<UtcTime>> {
        let query = sqlx::query("SELECT MAX(updated_at) from streams")
            .try_map(|row: PgRow| row.try_get::<Option<UtcTime>, _>("max"))
            .fetch_one(pool);

        crate::otel::instrument("SELECT", "streams", query).await
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    assert_eq!(YouTubeStreamsLastUpdated.execute(&pool).await?, None);

    sqlx::query!(
        r#"
INSERT INTO streams (platform, platform_id, title, channel_id, schedule_time, start_time, end_time, updated_at, status)
     VALUES ('youtube', 'id1', 'title1', 1, NULL, NULL, NULL, to_timestamp(1000), 'ended'),
            ('youtube', 'id2', 'title2', 2, NULL, NULL, NULL, to_timestamp(3000), 'ended'),
            ('youtube', 'id3', 'title3', 3, NULL, NULL, NULL, to_timestamp(2000), 'ended');
        "#
    )
    .execute(&pool)
    .await?;

    assert_eq!(
        YouTubeStreamsLastUpdated {}.execute(&pool).await?,
        Some(UtcTime::from_utc(
            NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(),
            Utc
        ))
    );

    Ok(())
}
