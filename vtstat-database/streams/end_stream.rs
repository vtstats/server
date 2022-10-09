use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

use super::StreamStatus;

/// if `stream_status = "scheduled"`, then delete it, otherwise:
/// change `stream_status` to `ended` and update `end_time`,
/// `title`, `start_time`, `schedule_time`, `like_max` if provided
#[derive(Default)]
pub struct EndStreamQuery<'q> {
    pub stream_id: i32,
    pub title: Option<&'q str>,
    pub end_time: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub schedule_time: Option<DateTime<Utc>>,
    pub likes: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl<'q> EndStreamQuery<'q> {
    pub async fn execute(&self, pool: &PgPool) -> Result<()> {
        let query = sqlx::query!(
            r#"SELECT status as "status: StreamStatus" FROM streams WHERE stream_id = $1"#,
            self.stream_id
        )
        .map(|r| r.status)
        .fetch_optional(pool);

        let status = crate::otel::instrument("SELECT", "youtube_streams", query).await?;

        match status {
            Some(StreamStatus::Scheduled) => {
                let query = sqlx::query!(
                    r#"DELETE FROM streams WHERE stream_id = $1"#,
                    self.stream_id, // $1
                )
                .execute(pool);

                crate::otel::instrument("DELETE", "streams", query).await?;
            }
            Some(_) => {
                let query = sqlx::query!(
                    r#"
                 UPDATE streams
                    SET status        = 'ended',
                        title         = COALESCE($1, title),
                        end_time      = COALESCE($2, end_time),
                        start_time    = COALESCE($3, start_time),
                        schedule_time = COALESCE($4, schedule_time),
                        like_max      = GREATEST($5, like_max),
                        updated_at    = COALESCE($6, updated_at)
                  WHERE stream_id     = $7
                        "#,
                    self.title,         // $1
                    self.end_time,      // $2
                    self.start_time,    // $3
                    self.schedule_time, // $4
                    self.likes,         // $5
                    self.updated_at,    // $6
                    self.stream_id,     // $7
                )
                .execute(pool);

                crate::otel::instrument("UPDATE", "streams", query).await?;
            }
            None => {
                tracing::debug!("Stream not found");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query!(
        r#"
    INSERT INTO streams (stream_id, platform, platform_id, title, like_max, status, channel_id)
         VALUES (1, 'youtube', 'id1', 'title1', 100, 'live', 1),
                (2, 'youtube', 'id2', 'title2', 100, 'scheduled', 1),
                (3, 'youtube', 'id3', 'title3', 100, 'ended', 1);
    "#
    )
    .execute(&pool)
    .await?;

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp(9000, 0), Utc);

        EndStreamQuery {
            stream_id: 1,
            end_time: Some(time),
            likes: Some(0),
            ..Default::default()
        }
        .execute(&pool)
        .await?;

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
        EndStreamQuery {
            stream_id: 2,
            ..Default::default()
        }
        .execute(&pool)
        .await?;

        let row = sqlx::query!(r#"SELECT title FROM streams WHERE stream_id = 2"#)
            .fetch_optional(&pool)
            .await?;

        assert!(row.is_none());
    }

    {
        EndStreamQuery {
            stream_id: 3,
            likes: Some(200),
            ..Default::default()
        }
        .execute(&pool)
        .await?;

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
