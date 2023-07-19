use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool, Result};

/// 1. change `stream_status` to `live` and
/// 2. update `start_time` if not set
/// 3. update `title` and `like_max` if provided
pub struct StartStreamQuery<'q> {
    pub stream_id: i32,
    pub start_time: DateTime<Utc>,
    pub likes: Option<i32>,
    pub title: Option<&'q str>,
}

impl<'q> StartStreamQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let query = sqlx::query(
            r#"
     UPDATE streams
        SET title          = COALESCE($1, title),
            updated_at     = $2,
            start_time     = COALESCE(start_time, $2),
            status         = 'live',
            like_max       = GREATEST($3, like_max)
      WHERE stream_id      = $4
            "#,
        )
        .bind(self.title) // $1
        .bind(self.start_time) // $2
        .bind(self.likes) // $3
        .bind(self.stream_id) // $4
        .execute(pool);

        crate::otel::instrument("UPDATE", "streams", query).await
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query!(
        r#"
INSERT INTO streams (stream_id, title, channel_id, platform_id, platform, schedule_time, start_time, end_time, status)
     VALUES (1, 'title1', 1, 'id1', 'youtube', to_timestamp(0), NULL, NULL, 'scheduled'),
            (2, 'title2', 2, 'id2', 'youtube', to_timestamp(0), to_timestamp(10000), to_timestamp(12000), 'live'),
            (3, 'title3', 2, 'id3', 'youtube', to_timestamp(10000), to_timestamp(15000), to_timestamp(17000), 'ended');
        "#
    )
    .execute(&pool)
    .await?;

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        let res = StartStreamQuery {
            stream_id: 1,
            start_time: time,
            likes: None,
            title: None,
        }
        .execute(&pool)
        .await?;

        let row = sqlx::query!("SELECT status::TEXT, start_time FROM streams WHERE stream_id = 1")
            .fetch_one(&pool)
            .await?;

        assert_eq!(res.rows_affected(), 1);
        assert_eq!(row.status, Some("live".into()));
        assert_eq!(row.start_time, Some(time));
    }

    {
        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        let res = StartStreamQuery {
            stream_id: 2,
            start_time: time,
            likes: Some(100),
            title: Some("title_alt"),
        }
        .execute(&pool)
        .await?;

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
