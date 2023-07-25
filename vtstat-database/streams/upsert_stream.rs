use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::StreamStatus;

/// insert or update a stream row
#[derive(Default)]
pub struct UpsertYouTubeStreamQuery<'q> {
    // TODO: add platform field
    pub platform_stream_id: &'q str,
    pub channel_id: i32,
    pub title: &'q str,
    pub status: StreamStatus,

    pub thumbnail_url: Option<String>,
    pub schedule_time: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl<'q> UpsertYouTubeStreamQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<i32> {
        let query = sqlx::query!(
            r#"
INSERT INTO streams AS t (
                platform,
                platform_id,
                channel_id,
                title,
                status,
                thumbnail_url,
                schedule_time,
                start_time,
                end_time
            )
     VALUES ('youtube', $1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (platform, platform_id) DO UPDATE
        SET title          = COALESCE($3, t.title),
            status         = COALESCE($4, t.status),
            thumbnail_url  = COALESCE($5, t.thumbnail_url),
            schedule_time  = COALESCE($6, t.schedule_time),
            start_time     = COALESCE($7, t.start_time),
            end_time       = COALESCE($8, t.end_time)
  RETURNING stream_id
            "#,
            self.platform_stream_id, // $1
            self.channel_id,         // $2
            self.title,              // $3
            self.status as _,        // $4
            self.thumbnail_url,      // $5
            self.schedule_time,      // $6
            self.start_time,         // $7
            self.end_time,           // $8
        )
        .fetch_one(pool);

        let record = crate::otel::instrument("INSERT", "streams", query).await?;

        Ok(record.stream_id)
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    {
        let rows = sqlx::query!(r#"SELECT title FROM streams WHERE channel_id = 1"#)
            .fetch_all(&pool)
            .await?;

        assert_eq!(rows.len(), 0);

        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        let stream_id = UpsertYouTubeStreamQuery {
            channel_id: 1,
            platform_stream_id: "id1",
            title: "title1",
            status: StreamStatus::Live,
            thumbnail_url: Some("http://bing.com".into()),
            start_time: Some(time),
            ..Default::default()
        }
        .execute(&pool)
        .await?;

        let rows = sqlx::query!(
            r#"SELECT title, start_time, status::TEXT FROM streams WHERE channel_id = 1"#
        )
        .fetch_all(&pool)
        .await?;

        assert_eq!(stream_id, 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "title1");
        assert_eq!(rows[0].status, Some("live".into()));
        assert_eq!(rows[0].start_time, Some(time));
    }

    {
        let stream_id = UpsertYouTubeStreamQuery {
            channel_id: 1,
            platform_stream_id: "id1",
            status: StreamStatus::Ended,
            title: "title2",
            thumbnail_url: Some("https://google.com".into()),
            ..Default::default()
        }
        .execute(&pool)
        .await?;

        let rows = sqlx::query!(
            r#"SELECT title, status::TEXT, start_time, thumbnail_url FROM streams WHERE channel_id = 1"#
        )
        .fetch_all(&pool)
        .await?;

        assert_eq!(stream_id, 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].thumbnail_url, Some("https://google.com".into()));
        assert_eq!(rows[0].status, Some("ended".to_string()));
        assert_eq!(rows[0].title, "title2");
        assert_eq!(
            rows[0].start_time,
            Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(),
                Utc
            ))
        );
    }

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        let stream_id = UpsertYouTubeStreamQuery {
            channel_id: 1,
            platform_stream_id: "id1",
            start_time: Some(time),
            ..Default::default()
        }
        .execute(&pool)
        .await?;

        let rows = sqlx::query!(r#"SELECT start_time FROM streams WHERE channel_id = 1"#)
            .fetch_all(&pool)
            .await?;

        assert_eq!(stream_id, 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].start_time, Some(time));
    }

    Ok(())
}
