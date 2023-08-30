use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};

pub struct AddStreamViewerStatsQuery {
    pub stream_id: i32,
    pub time: DateTime<Utc>,
    pub count: i32,
}

impl AddStreamViewerStatsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<()> {
        let query = sqlx::query!(
            r#"
INSERT INTO stream_viewer_stats AS s (stream_id, time, count)
     VALUES ($1, $2, $3)
ON CONFLICT (stream_id, time) DO UPDATE
        SET count = GREATEST(excluded.count, s.count)
            "#,
            self.stream_id,
            self.time,
            self.count,
        )
        .execute(pool);

        crate::otel::execute_query!("INSERT", "stream_viewer_stats", query)?;

        let query = sqlx::query!(
            r#"
     UPDATE streams
        SET viewer_max = GREATEST(viewer_max, $1),
            viewer_avg = (SELECT AVG(count) FROM stream_viewer_stats WHERE stream_id = $2),
            updated_at = $3
      WHERE stream_id = $2
            "#,
            self.count,
            self.stream_id,
            self.time,
        )
        .execute(pool);

        crate::otel::execute_query!("UPDATE", "streams", query)?;

        Ok(())
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::{Duration, NaiveDateTime};

    let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(9000, 0).unwrap(), Utc);

    AddStreamViewerStatsQuery {
        stream_id: 1,
        time,
        count: 40,
    }
    .execute(&pool)
    .await?;

    AddStreamViewerStatsQuery {
        stream_id: 1,
        time: time + Duration::seconds(15),
        count: 20,
    }
    .execute(&pool)
    .await?;

    AddStreamViewerStatsQuery {
        stream_id: 1,
        time,
        count: 20,
    }
    .execute(&pool)
    .await?;

    let stats = sqlx::query!("SELECT * FROM stream_viewer_stats ORDER BY time ASC")
        .fetch_all(&pool)
        .await?;

    assert_eq!(stats.len(), 2);
    assert_eq!(stats[0].count, 40);
    assert_eq!(stats[1].count, 20);

    Ok(())
}
