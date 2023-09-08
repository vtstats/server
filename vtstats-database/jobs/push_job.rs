use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, Result};

use super::*;

/// push a new job into queue
pub struct PushJobQuery {
    pub payload: JobPayload,
    pub next_run: Option<DateTime<Utc>>,
    pub continuation: Option<String>,
}

impl PushJobQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<i32> {
        let query = sqlx::query!(
            r#"
INSERT INTO jobs (kind, payload, status, next_run, continuation)
     VALUES ($1, $2, 'queued', $3, $4)
ON CONFLICT (kind, payload) DO UPDATE
        SET status       = 'queued',
            next_run     = $3,
            continuation = $4,
            updated_at   = NOW()
  RETURNING job_id
            "#,
            self.payload.kind() as _, // $1
            Json(self.payload) as _,  // $2
            self.next_run,            // $3
            self.continuation,        // $4
        )
        .fetch_one(pool);

        let record = crate::otel::execute_query!("INSERT", "jobs", query)?;

        if let Some(next_run) = self.next_run {
            let _ = sqlx::query!(
                "SELECT pg_notify('vt_new_job_queued', $1)",
                next_run.timestamp_millis().to_string()
            )
            .execute(pool)
            .await
            .map_err(|err| {
                tracing::error!("push job: {err:?}");
            });
        }

        Ok(record.job_id)
    }
}

pub async fn queue_send_notification(
    time: DateTime<Utc>,
    stream_platform: String,
    stream_platform_id: String,
    vtuber_id: String,
    pool: &PgPool,
) -> Result<i32> {
    PushJobQuery {
        continuation: None,
        next_run: Some(time),
        payload: JobPayload::SendNotification(SendNotificationJobPayload {
            stream_platform,
            stream_platform_id,
            vtuber_id,
        }),
    }
    .execute(pool)
    .await
}

pub async fn queue_collect_youtube_stream_metadata(
    time: DateTime<Utc>,
    stream_id: i32,
    platform_stream_id: String,
    platform_channel_id: String,
    pool: &PgPool,
) -> Result<i32> {
    PushJobQuery {
        continuation: None,
        next_run: Some(time),
        payload: JobPayload::CollectYoutubeStreamMetadata(CollectYoutubeStreamMetadataJobPayload {
            stream_id,
            platform_stream_id,
            platform_channel_id,
        }),
    }
    .execute(pool)
    .await
}

pub async fn queue_collect_twitch_stream_metadata(
    time: DateTime<Utc>,
    stream_id: i32,
    platform_stream_id: String,
    platform_channel_id: String,
    platform_channel_login: String,
    pool: &PgPool,
) -> Result<i32> {
    PushJobQuery {
        continuation: None,
        next_run: Some(time),
        payload: JobPayload::CollectTwitchStreamMetadata(CollectTwitchStreamMetadataJobPayload {
            stream_id,
            platform_stream_id,
            platform_channel_id,
            platform_channel_login,
        }),
    }
    .execute(pool)
    .await
}

#[cfg(test)]
#[sqlx::test]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query("DELETE FROM jobs").execute(&pool).await?;

    // should push a new job
    {
        PushJobQuery {
            continuation: None,
            next_run: None,
            payload: JobPayload::HealthCheck,
        }
        .execute(&pool)
        .await?;

        PushJobQuery {
            continuation: Some("continuation".into()),
            next_run: None,
            payload: JobPayload::UpsertYoutubeStream(UpsertYoutubeStreamJobPayload {
                vtuber_id: "poi".into(),
                channel_id: 0,
                platform_stream_id: "foo".into(),
            }),
        }
        .execute(&pool)
        .await?;

        let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(3000, 0).unwrap(), Utc);

        PushJobQuery {
            continuation: None,
            next_run: Some(time),
            payload: JobPayload::UpsertYoutubeStream(UpsertYoutubeStreamJobPayload {
                vtuber_id: "poi".into(),
                channel_id: 0,
                platform_stream_id: "bar".into(),
            }),
        }
        .execute(&pool)
        .await?;

        assert_eq!(
            sqlx::query!("SELECT COUNT(*) as \"count!\" FROM jobs")
                .fetch_one(&pool)
                .await?
                .count,
            3
        );
    }

    // re-queued jobs without same payload
    {
        sqlx::query("UPDATE jobs SET status = 'success'")
            .execute(&pool)
            .await?;

        PushJobQuery {
            continuation: Some("foobar".into()),
            next_run: None,
            payload: JobPayload::UpsertYoutubeStream(UpsertYoutubeStreamJobPayload {
                vtuber_id: "poi".into(),
                channel_id: 0,
                platform_stream_id: "foo".into(),
            }),
        }
        .execute(&pool)
        .await?;

        assert_eq!(
            sqlx::query_as::<_, Job>("SELECT * FROM jobs")
                .fetch_all(&pool)
                .await?
                .len(),
            3
        );
        assert_eq!(
            sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE status = 'queued'")
                .fetch_all(&pool)
                .await?
                .len(),
            1
        );
    }

    Ok(())
}
