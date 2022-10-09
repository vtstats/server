use sqlx::{PgPool, Result};

pub struct GetUpcomingStreamsQuery;

#[derive(Debug, PartialEq, Eq)]
pub struct UpcomingStream {
    pub stream_id: i32,
    pub platform_stream_id: String,
    pub platform_channel_id: String,
}

impl GetUpcomingStreamsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<UpcomingStream>> {
        let query = sqlx::query_as!(
            UpcomingStream,
            r#"
     SELECT stream_id, s.platform_id AS platform_stream_id, c.platform_id AS "platform_channel_id!"
       FROM streams s
  LEFT JOIN channels c ON c.channel_id = s.channel_id
      WHERE end_time IS NULL
        AND (
              start_time IS NOT NULL OR (
                schedule_time > NOW() - INTERVAL '6 hours'
                and schedule_time < NOW() + INTERVAL '5 minutes'
              )
            )
            "#
        )
        .fetch_all(pool);

        crate::otel::instrument("SELECT", "streams", query).await
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    sqlx::query!(
        r#"        
INSERT INTO streams (stream_id, platform, platform_id, title, channel_id, schedule_time, start_time, end_time, status)
     VALUES (1, 'youtube', 'id1', 'title1', 1, NULL, NOW() - INTERVAL '30m', NULL, 'live'),
            (2, 'youtube', 'id2', 'title2', 1, NULL, NULL, NOW() - INTERVAL '30m', 'live'),
            (3, 'youtube', 'id3', 'title3', 1, NOW() + INTERVAL '4m', NULL, NULL, 'live'),
            (4, 'youtube', 'id4', 'title4', 1, NOW() + INTERVAL '6m', NULL, NULL, 'live'),
            (5, 'youtube', 'id5', 'title5', 1, NOW() - INTERVAL '5h', NULL, NULL, 'live'),
            (6, 'youtube', 'id6', 'title6', 1, NOW() - INTERVAL '7h', NULL, NULL, 'live');
        "#
    )
    .execute(&pool)
    .await?;

    let streams = GetUpcomingStreamsQuery.execute(&pool).await?;

    assert_eq!(
        streams,
        vec![
            UpcomingStream {
                stream_id: 1,
                platform_stream_id: "id1".into(),
                platform_channel_id: "platform_channel_id1".into(),
            },
            UpcomingStream {
                stream_id: 3,
                platform_stream_id: "id3".into(),
                platform_channel_id: "platform_channel_id1".into(),
            },
            UpcomingStream {
                stream_id: 5,
                platform_stream_id: "id5".into(),
                platform_channel_id: "platform_channel_id1".into(),
            },
        ]
    );

    Ok(())
}
