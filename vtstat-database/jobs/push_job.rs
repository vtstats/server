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
    pub async fn execute(self, pool: &PgPool) -> Result<Job> {
        let kind = match self.payload {
            JobPayload::HealthCheck => JobKind::HealthCheck,
            JobPayload::RefreshYoutubeRss => JobKind::RefreshYoutubeRss,
            JobPayload::SubscribeYoutubePubsub => JobKind::SubscribeYoutubePubsub,
            JobPayload::UpdateYoutubeChannelViewAndSubscriber => {
                JobKind::UpdateYoutubeChannelViewAndSubscriber
            }
            JobPayload::UpdateBilibiliChannelViewAndSubscriber => {
                JobKind::UpdateBilibiliChannelViewAndSubscriber
            }
            JobPayload::UpdateYoutubeChannelDonation => JobKind::UpdateYoutubeChannelDonation,
            JobPayload::UpdateCurrencyExchangeRate => JobKind::UpdateCurrencyExchangeRate,
            JobPayload::UpsertYoutubeStream(_) => JobKind::UpsertYoutubeStream,
            JobPayload::CollectYoutubeStreamMetadata(_) => JobKind::CollectYoutubeStreamMetadata,
            JobPayload::CollectYoutubeStreamLiveChat(_) => JobKind::CollectYoutubeStreamLiveChat,
            JobPayload::UpdateUpcomingStream => JobKind::UpdateUpcomingStream,
            JobPayload::SendNotification(_) => JobKind::SendNotification,
            JobPayload::InstallDiscordCommands => JobKind::InstallDiscordCommands,
        };

        let query = sqlx::query_as::<_, Job>(
            r#"
INSERT INTO jobs (kind, payload, status, next_run, continuation)
     VALUES ($1, $2, 'queued', $3, $4)
ON CONFLICT (kind, payload) DO UPDATE
        SET status       = 'queued',
            next_run     = $3,
            continuation = $4,
            updated_at   = NOW()
  RETURNING *
            "#,
        )
        .bind(kind)
        .bind(Json(self.payload))
        .bind(self.next_run)
        .bind(self.continuation)
        .fetch_one(pool);

        crate::otel::instrument("INSERT", "jobs", query).await
    }
}

#[cfg(test)]
#[sqlx::test]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    sqlx::query("DELETE FROM jobs").execute(&pool).await?;

    // should push a new job
    {
        let job = PushJobQuery {
            continuation: None,
            next_run: None,
            payload: JobPayload::HealthCheck,
        }
        .execute(&pool)
        .await?;

        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.payload, JobPayload::HealthCheck);
        assert_eq!(job.next_run, None);
        assert_eq!(job.continuation, None);

        let job = PushJobQuery {
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

        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.next_run, None);
        assert_eq!(job.continuation, Some("continuation".into()));

        let time = DateTime::from_utc(NaiveDateTime::from_timestamp(3000, 0), Utc);

        let job = PushJobQuery {
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

        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.next_run, Some(time));
        assert_eq!(job.continuation, None);

        assert_eq!(
            sqlx::query_as::<_, Job>("SELECT * FROM jobs")
                .fetch_all(&pool)
                .await?
                .len(),
            3
        );
    }

    // re-queued jobs without same payload
    {
        sqlx::query("UPDATE jobs SET status = 'success'")
            .execute(&pool)
            .await?;

        let job = PushJobQuery {
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

        assert_eq!(job.continuation, Some("foobar".into()));

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
