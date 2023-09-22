use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder, Result};

pub struct AddChannelSubscriberStatsQuery<I: IntoIterator<Item = AddChannelSubscriberStatsRow>> {
    pub time: DateTime<Utc>,
    pub rows: I,
}

#[derive(Debug)]
pub struct AddChannelSubscriberStatsRow {
    pub channel_id: i32,
    pub value: i32,
}

impl<I: IntoIterator<Item = AddChannelSubscriberStatsRow>> AddChannelSubscriberStatsQuery<I> {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO channel_subscriber_stats AS s (channel_id, time, count) ",
        );

        query_builder.push_values(self.rows.into_iter(), |mut b, row| {
            b.push_bind(row.channel_id)
                .push_bind(self.time)
                .push_bind(row.value);
        });

        query_builder.push(
            "ON CONFLICT (channel_id, time) DO UPDATE \
            SET count = GREATEST(excluded.count, s.count)",
        );

        let query = query_builder.build().execute(pool);

        crate::otel::execute_query!("INSERT", "channel_subscriber_stats", query)
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::TimeZone;
    use sqlx::Row;

    let time = Utc.timestamp_opt(9000, 0).single().unwrap();

    let result = AddChannelSubscriberStatsQuery {
        time,
        rows: [
            AddChannelSubscriberStatsRow {
                channel_id: 1,
                value: 20,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 2,
                value: 10,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 3,
                value: 20,
            },
        ],
    }
    .execute(&pool)
    .await?;

    assert_eq!(result.rows_affected(), 3);
    let stats = sqlx::query("SELECT count v1, time ts FROM channel_subscriber_stats")
        .fetch_all(&pool)
        .await?;
    assert_eq!(stats.len(), 3);

    let result = AddChannelSubscriberStatsQuery {
        time,
        rows: [
            AddChannelSubscriberStatsRow {
                channel_id: 2,
                value: 20,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 3,
                value: 10,
            },
            AddChannelSubscriberStatsRow {
                channel_id: 4,
                value: 20,
            },
        ],
    }
    .execute(&pool)
    .await?;

    assert_eq!(result.rows_affected(), 3);
    let stats = sqlx::query("SELECT time, count FROM channel_subscriber_stats")
        .fetch_all(&pool)
        .await?;
    assert_eq!(stats.len(), 4);
    assert!(stats.iter().all(|s| s.get::<i32, _>("count") == 20));
    assert!(stats
        .iter()
        .all(|s| s.get::<DateTime<Utc>, _>("time") == time));

    Ok(())
}
