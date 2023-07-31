use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder, Result};

pub struct AddChannelSubscriberStatsQuery<'q> {
    pub time: DateTime<Utc>,
    pub rows: &'q [AddChannelSubscriberStatsRow],
}

pub struct AddChannelSubscriberStatsRow {
    pub channel_id: i32,
    pub count: i32,
}

impl<'q> AddChannelSubscriberStatsQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO channel_subscriber_stats AS s (channel_id, time, count) ",
        );

        query_builder.push_values(self.rows.iter(), |mut b, row| {
            b.push_bind(row.channel_id)
                .push_bind(self.time)
                .push_bind(row.count);
        });

        query_builder.push(
            "ON CONFLICT (channel_id, time) DO UPDATE \
            SET count = GREATEST(excluded.count, s.count)",
        );

        let query = query_builder.build().execute(pool);

        crate::otel::instrument("INSERT", "channel_subscriber_stats", query).await
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    let time = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(9000, 0).unwrap(), Utc);

    let result = AddChannelSubscriberStatsQuery {
        time,
        rows: &vec![
            AddChannelSubscriberStatsRow {
                channel_id: 1,
                count: 20,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 2,
                count: 10,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 3,
                count: 20,
            },
        ],
    }
    .execute(&pool)
    .await?;

    assert_eq!(result.rows_affected(), 3);
    let stats = sqlx::query!("SELECT time, count FROM channel_subscriber_stats")
        .fetch_all(&pool)
        .await?;
    assert_eq!(stats.len(), 3);

    let result = AddChannelSubscriberStatsQuery {
        time,
        rows: &vec![
            AddChannelSubscriberStatsRow {
                channel_id: 2,
                count: 20,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 3,
                count: 10,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 4,
                count: 20,
            },
        ],
    }
    .execute(&pool)
    .await?;

    assert_eq!(result.rows_affected(), 3);
    let stats = sqlx::query!("SELECT time, count FROM channel_subscriber_stats")
        .fetch_all(&pool)
        .await?;
    assert_eq!(stats.len(), 4);
    assert!(stats.iter().all(|s| s.count == 20));
    assert!(stats.iter().all(|s| s.time == time));

    Ok(())
}
