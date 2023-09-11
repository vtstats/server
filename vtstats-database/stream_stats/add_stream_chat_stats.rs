use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder, Result};

pub struct AddStreamChatStatsQuery {
    pub stream_id: i32,
    pub rows: Vec<AddStreamChatStatsRow>,
}

pub struct AddStreamChatStatsRow {
    pub time: DateTime<Utc>,
    pub count: i32,
    pub from_member_count: i32,
}

impl AddStreamChatStatsQuery {
    pub async fn execute(self, pool: &PgPool) -> Result<PgQueryResult> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO stream_chat_stats AS s (stream_id, time, count, from_member_count) ",
        );

        query_builder.push_values(self.rows.iter(), |mut b, row| {
            b.push_bind(self.stream_id)
                .push_bind(row.time)
                .push_bind(row.count)
                .push_bind(row.from_member_count);
        });

        query_builder.push(
            "ON CONFLICT (stream_id, time) DO UPDATE \
            SET count = excluded.count + s.count, \
            from_member_count = excluded.from_member_count + s.from_member_count",
        );

        let query = query_builder.build().execute(pool);

        crate::otel::execute_query!("INSERT", "stream_chat_stats", query)
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::{Duration, TimeZone};

    let time = Utc.timestamp_opt(9000, 0).single().unwrap();

    let result = AddStreamChatStatsQuery {
        stream_id: 1,
        rows: vec![
            AddStreamChatStatsRow {
                time,
                count: 70,
                from_member_count: 30,
            },
            AddStreamChatStatsRow {
                time: time + Duration::seconds(15),
                count: 40,
                from_member_count: 20,
            },
            AddStreamChatStatsRow {
                time: time + Duration::seconds(30),
                count: 35,
                from_member_count: 15,
            },
        ],
    }
    .execute(&pool)
    .await?;

    assert_eq!(result.rows_affected(), 3);
    let stats = sqlx::query!("SELECT * FROM stream_chat_stats")
        .fetch_all(&pool)
        .await?;
    assert_eq!(stats.len(), 3);
    assert!(stats.iter().all(|s| s.time >= time));

    let result = AddStreamChatStatsQuery {
        stream_id: 1,
        rows: vec![
            AddStreamChatStatsRow {
                time: time + Duration::seconds(15),
                count: 30,
                from_member_count: 10,
            },
            AddStreamChatStatsRow {
                time: time + Duration::seconds(30),
                count: 35,
                from_member_count: 15,
            },
        ],
    }
    .execute(&pool)
    .await?;

    assert_eq!(result.rows_affected(), 2);
    let stats = sqlx::query!("SELECT * FROM stream_chat_stats")
        .fetch_all(&pool)
        .await?;
    assert_eq!(stats.len(), 3);
    assert!(stats.iter().all(|s| s.count == 70));
    assert!(stats.iter().all(|s| s.from_member_count == 30));
    assert!(stats.iter().all(|s| s.time >= time));

    Ok(())
}
