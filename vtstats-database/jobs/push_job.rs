use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, Result};

use super::*;

/// push a new job into queue
pub struct PushJobQuery {
    pub payload: JobPayload,
    pub next_run: Option<DateTime<Utc>>,
}

impl PushJobQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<i32> {
        struct Record {
            job_id: i32,
            status: JobStatus,
        }

        let query = sqlx::query_as!(
            Record,
            "INSERT INTO jobs as j (kind, payload, status, next_run) \
            VALUES ($1, $2, 'queued', $3) \
            ON CONFLICT (kind, payload) DO UPDATE \
            SET status = CASE WHEN j.status != 'running' THEN 'queued'::job_status ELSE 'running'::job_status END, \
            next_run = $3, updated_at = NOW() \
            RETURNING job_id, status as \"status: _\"",
            self.payload.kind() as _,  // $1
            Json(&self.payload) as _,  // $2
            self.next_run,             // $3
        )
        .fetch_one(pool);

        let record = crate::otel::execute_query!("INSERT", "jobs", query)?;

        if record.status == JobStatus::Running {
            tracing::warn!(
                "Same job already running kind={:?} payload={:?} job_id={}",
                self.payload.kind(),
                self.payload,
                record.job_id
            );
            return Ok(record.job_id);
        }

        if let Some(next_run) = self.next_run {
            let query = sqlx::query!(
                "SELECT pg_notify('vt_new_job_queued', $1)",
                next_run.timestamp_millis().to_string()
            )
            .execute(pool);

            let _ = crate::otel::execute_query!("SELECT", "pg_notify", query).map_err(|err| {
                tracing::error!("push job: {err:?}");
            });
        }

        Ok(record.job_id)
    }
}

pub async fn queue_send_notification(
    time: DateTime<Utc>,
    stream_id: i32,
    pool: &PgPool,
) -> Result<i32> {
    PushJobQuery {
        next_run: Some(time),
        payload: JobPayload::SendNotification(SendNotificationJobPayload { stream_id }),
    }
    .execute(pool)
    .await
}

pub async fn queue_collect_youtube_stream_metadata(
    time: DateTime<Utc>,
    stream_id: i32,
    pool: &PgPool,
) -> Result<i32> {
    PushJobQuery {
        next_run: Some(time),
        payload: JobPayload::CollectYoutubeStreamMetadata(CollectYoutubeStreamMetadataJobPayload {
            stream_id,
        }),
    }
    .execute(pool)
    .await
}

pub async fn queue_collect_twitch_stream_metadata(
    time: DateTime<Utc>,
    stream_id: i32,
    pool: &PgPool,
) -> Result<i32> {
    PushJobQuery {
        next_run: Some(time),
        payload: JobPayload::CollectTwitchStreamMetadata(CollectTwitchStreamMetadataJobPayload {
            stream_id,
        }),
    }
    .execute(pool)
    .await
}

#[cfg(test)]
#[sqlx::test]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::TimeZone;

    sqlx::query("DELETE FROM jobs").execute(&pool).await?;

    // should push a new job
    {
        PushJobQuery {
            next_run: None,
            payload: JobPayload::HealthCheck,
        }
        .execute(&pool)
        .await?;

        PushJobQuery {
            next_run: None,
            payload: JobPayload::CollectTwitchStreamMetadata(
                CollectTwitchStreamMetadataJobPayload { stream_id: 0 },
            ),
        }
        .execute(&pool)
        .await?;

        let time = Utc.timestamp_opt(3000, 0).single().unwrap();

        PushJobQuery {
            next_run: Some(time),
            payload: JobPayload::CollectTwitchStreamMetadata(
                CollectTwitchStreamMetadataJobPayload { stream_id: 1 },
            ),
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

    // re-queued jobs w/ same payload
    {
        sqlx::query("UPDATE jobs SET status = 'success'")
            .execute(&pool)
            .await?;

        PushJobQuery {
            next_run: None,
            payload: JobPayload::CollectTwitchStreamMetadata(
                CollectTwitchStreamMetadataJobPayload { stream_id: 0 },
            ),
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

    // re-queued jobs that is already running same payload
    {
        sqlx::query("UPDATE jobs SET status = 'running'")
            .execute(&pool)
            .await?;

        PushJobQuery {
            next_run: None,
            payload: JobPayload::CollectTwitchStreamMetadata(
                CollectTwitchStreamMetadataJobPayload { stream_id: 0 },
            ),
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
            0
        );
    }

    Ok(())
}
