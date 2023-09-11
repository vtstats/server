use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Result};

use chrono::serde::{ts_milliseconds, ts_milliseconds_option};
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::channels::Platform;

type UtcTime = DateTime<Utc>;

#[skip_serializing_none]
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub platform: Platform,
    pub platform_id: String,
    pub stream_id: i32,
    pub channel_id: i32,
    pub title: String,
    pub highlighted_title: Option<String>,
    pub vtuber_id: String,
    pub thumbnail_url: Option<String>,
    #[serde(with = "ts_milliseconds_option")]
    pub schedule_time: Option<UtcTime>,
    #[serde(with = "ts_milliseconds_option")]
    pub start_time: Option<UtcTime>,
    #[serde(with = "ts_milliseconds_option")]
    pub end_time: Option<UtcTime>,
    pub viewer_avg: Option<i32>,
    pub viewer_max: Option<i32>,
    pub like_max: Option<i32>,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: UtcTime,
    pub status: StreamStatus,
}

#[derive(Debug, sqlx::Type, Serialize, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "stream_status", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum StreamStatus {
    #[default]
    Scheduled,
    Live,
    Ended,
}

#[derive(Debug)]
pub enum Column {
    StartTime,
    EndTime,
    ScheduleTime,
    UpdatedAt,
}

impl Column {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Column::ScheduleTime => "schedule_time",
            Column::EndTime => "end_time",
            Column::StartTime => "start_time",
            Column::UpdatedAt => "updated_at",
        }
    }
}

#[derive(Debug)]
pub enum Ordering {
    Asc,
    Desc,
}

impl Ordering {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Ordering::Asc => "ASC",
            Ordering::Desc => "DESC",
        }
    }
}

pub async fn filter_streams_order_by_schedule_time_asc(
    channel_ids: &[i32],
    status: StreamStatus,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    pool: PgPool,
) -> Result<Vec<Stream>> {
    let query = sqlx::query_as!(
        Stream,
        "SELECT platform as \"platform: _\", \
        platform_id, \
        stream_id, \
        title, \
        channel_id, \
        null as highlighted_title, \
        vtuber_id, \
        thumbnail_url, \
        schedule_time, \
        start_time, \
        end_time, \
        viewer_max, \
        viewer_avg, \
        like_max, \
        updated_at, \
        status as \"status: _\" \
        FROM streams \
        WHERE channel_id = ANY($1) \
        AND status = $2 \
        AND (schedule_time > $3 OR $3 IS NULL) \
        AND (schedule_time < $4 OR $4 IS NULL) \
        ORDER BY schedule_time ASC \
        LIMIT 24",
        channel_ids,
        status as _,
        start_at,
        end_at
    )
    .fetch_all(&pool);

    crate::otel::execute_query!("SELECT", "streams", query)
}

pub async fn filter_streams_order_by_start_time_desc(
    channel_ids: &[i32],
    status: StreamStatus,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    keyword: Option<&str>,
    pool: PgPool,
) -> Result<Vec<Stream>> {
    if let Some(keyword) = keyword {
        let query = sqlx::query!(
            "SELECT pgroonga_query_expand('pgroonga_synonyms', 'term', 'synonyms', $1) as expended",
            keyword
        )
        .fetch_one(&pool);

        let expended = crate::otel::execute_query!("SELECT", "pgroonga_query_expand", query)?;

        let keyword = expended.expended.as_deref().unwrap_or(keyword);

        let query = sqlx::query_as!(
            Stream,
            "SELECT platform as \"platform: _\", \
            platform_id, \
            stream_id, \
            title, \
            channel_id, \
            pgroonga_highlight_html(\
                title, pgroonga_query_extract_keywords($5)\
            ) as highlighted_title, \
            vtuber_id, \
            thumbnail_url, \
            schedule_time, \
            start_time, \
            end_time, \
            viewer_max, \
            viewer_avg, \
            like_max, \
            updated_at, \
            status as \"status: _\" \
            FROM streams \
            WHERE channel_id = ANY($1) \
            AND status = $2 \
            AND (start_time > $3 OR $3 IS NULL) \
            AND (start_time < $4 OR $4 IS NULL) \
            AND title &@~ $5 \
            ORDER BY start_time DESC \
            LIMIT 24",
            channel_ids,
            status as _,
            start_at,
            end_at,
            keyword
        )
        .fetch_all(&pool);

        crate::otel::execute_query!("SELECT", "streams", query)
    } else {
        let query = sqlx::query_as!(
            Stream,
            "SELECT platform as \"platform: _\", \
            platform_id, \
            stream_id, \
            title, \
            channel_id, \
            null as highlighted_title, \
            vtuber_id, \
            thumbnail_url, \
            schedule_time, \
            start_time, \
            end_time, \
            viewer_max, \
            viewer_avg, \
            like_max, \
            updated_at, \
            status as \"status: _\" \
            FROM streams \
            WHERE channel_id = ANY($1) \
            AND status = $2 \
            AND (start_time > $3 OR $3 IS NULL) \
            AND (start_time < $4 OR $4 IS NULL) \
            ORDER BY start_time DESC \
            LIMIT 24",
            channel_ids,
            status as _,
            start_at,
            end_at
        )
        .fetch_all(&pool);

        crate::otel::execute_query!("SELECT", "streams", query)
    }
}

pub struct ListYouTubeStreamsQuery<'q> {
    pub ids: &'q [i32],
    // TODO add platform
    pub vtuber_ids: &'q [String],
    pub platform_ids: &'q [String],
    pub status: &'q [String],
    pub order_by: Option<(Column, Ordering)>,
    pub start_at: Option<(Column, &'q UtcTime)>,
    pub end_at: Option<(Column, &'q UtcTime)>,
    pub keyword: Option<&'q str>,
    pub limit: Option<usize>,
}

impl<'q> Default for ListYouTubeStreamsQuery<'q> {
    fn default() -> Self {
        ListYouTubeStreamsQuery {
            ids: &[],
            vtuber_ids: &[],
            platform_ids: &[],
            status: &[],
            order_by: None,
            end_at: None,
            start_at: None,
            keyword: None,
            limit: Some(24),
        }
    }
}

impl<'q> ListYouTubeStreamsQuery<'q> {
    pub async fn execute(self, pool: &PgPool) -> Result<Vec<Stream>> {
        let mut query_builder = self.into_query_builder();

        let query = query_builder.build_query_as::<Stream>().fetch_all(pool);

        crate::otel::execute_query!("SELECT", "youtube_streams", query)
    }

    pub fn into_query_builder(self) -> QueryBuilder<'q, Postgres> {
        let init = "\
       SELECT platform, \
              platform_id, \
              stream_id, \
              title, \
              channel_id, \
              null as highlighted_title, \
              vtuber_id, \
              thumbnail_url, \
              schedule_time, \
              start_time, \
              end_time, \
              viewer_max, \
              viewer_avg, \
              like_max, \
              updated_at, \
              status \
         FROM streams s\
         ";

        let mut qb = QueryBuilder::<Postgres>::new(init);

        let mut word = " WHERE ";

        if !self.ids.is_empty() {
            qb.push(word);
            word = " AND ";
            qb.push("stream_id = ANY(");
            qb.push_bind(self.ids);
            qb.push(")");
        }

        if !self.vtuber_ids.is_empty() {
            qb.push(word);
            word = " AND ";
            qb.push("vtuber_id = ANY(");
            qb.push_bind(self.vtuber_ids);
            qb.push(")");
        }

        if !self.platform_ids.is_empty() {
            qb.push(word);
            word = " AND ";
            qb.push("s.platform_id = ANY(");
            qb.push_bind(self.platform_ids);
            qb.push(")");
        }

        if !self.status.is_empty() {
            qb.push(word);
            word = " AND ";
            qb.push("status::TEXT = ANY(");
            qb.push_bind(self.status);
            qb.push(")");
        }

        if let Some((column, start_at)) = self.start_at {
            qb.push(word);
            word = " AND ";
            qb.push(format_args!("{} > ", column.as_str()));
            qb.push_bind(start_at);
        }

        if let Some((column, end_at)) = self.end_at {
            qb.push(word);
            qb.push(format_args!("{} < ", column.as_str()));
            qb.push_bind(end_at);
        }

        if let Some((column, ordering)) = self.order_by {
            qb.push(format_args!(
                " ORDER BY {} {}",
                column.as_str(),
                ordering.as_str()
            ));
        }

        if let Some(limit) = self.limit {
            qb.push(" LIMIT ");
            qb.push(limit.to_string());
        }

        qb
    }
}

#[cfg(test)]
#[sqlx::test(fixtures("channels"))]
async fn test(pool: PgPool) -> Result<()> {
    use chrono::NaiveDateTime;

    assert!(ListYouTubeStreamsQuery {
        vtuber_ids: &["poi".into()],
        order_by: Some((Column::StartTime, Ordering::Asc)),
        ..Default::default()
    }
    .into_query_builder()
    .sql()
    .ends_with(
        "WHERE vtuber_id = ANY($1) \
        ORDER BY start_time ASC \
        LIMIT 24"
    ),);

    assert!(ListYouTubeStreamsQuery {
        vtuber_ids: &["poi".into()],
        order_by: Some((Column::StartTime, Ordering::Asc)),
        ..Default::default()
    }
    .into_query_builder()
    .sql()
    .ends_with(
        "WHERE vtuber_id = ANY($1) \
            ORDER BY start_time ASC \
            LIMIT 24"
    ),);

    assert!(ListYouTubeStreamsQuery {
        vtuber_ids: &["poi".into()],
        order_by: Some((Column::EndTime, Ordering::Asc)),
        start_at: Some((Column::EndTime, &Utc::now())),
        ..Default::default()
    }
    .into_query_builder()
    .sql()
    .ends_with(
        "WHERE vtuber_id = ANY($1) \
        AND end_time > $2 \
        ORDER BY end_time ASC \
        LIMIT 24"
    ),);

    assert!(ListYouTubeStreamsQuery {
        vtuber_ids: &["poi".into()],
        order_by: Some((Column::ScheduleTime, Ordering::Desc)),
        start_at: Some((Column::ScheduleTime, &Utc::now())),
        end_at: Some((Column::ScheduleTime, &Utc::now())),
        limit: Some(2434),
        ..Default::default()
    }
    .into_query_builder()
    .sql()
    .ends_with(
        "WHERE vtuber_id = ANY($1) \
        AND schedule_time > $2 \
        AND schedule_time < $3 \
        ORDER BY schedule_time DESC \
        LIMIT 2434"
    ),);

    assert!(ListYouTubeStreamsQuery {
        vtuber_ids: &["poi".into()],
        limit: None,
        ..Default::default()
    }
    .into_query_builder()
    .sql()
    .ends_with("WHERE vtuber_id = ANY($1)"),);

    assert!(ListYouTubeStreamsQuery {
        platform_ids: &["poi".into()],
        ..Default::default()
    }
    .into_query_builder()
    .sql()
    .ends_with(
        "WHERE s.platform_id = ANY($1) \
        LIMIT 24"
    ),);

    sqlx::query!(
        r#"
INSERT INTO streams (stream_id, vtuber_id, title, channel_id, platform_id, platform, schedule_time, start_time, end_time, status)
     VALUES (1, 'vtuber1', 'title1', 1, 'id1', 'youtube', to_timestamp(200),   to_timestamp(1800),  to_timestamp(8000),  'live'),
            (2, 'vtuber2', 'title2', 2, 'id2', 'youtube', to_timestamp(0),     to_timestamp(10000), to_timestamp(12000), 'live'),
            (3, 'vtuber3', 'title3', 2, 'id3', 'youtube', to_timestamp(10000), to_timestamp(15000), to_timestamp(17000), 'ended');
        "#
    )
    .execute(&pool)
    .await?;

    assert_eq!(
        ListYouTubeStreamsQuery {
            ids: &[1, 2],
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            vtuber_ids: &["vtuber2".into(), "vtuber3".into()],
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            limit: Some(2),
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            status: &["live".into(), "scheduled".into()],
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            platform_ids: &["id1".into(), "id2".into(), "id4".into()],
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            start_at: Some((
                Column::StartTime,
                &UtcTime::from_utc(NaiveDateTime::from_timestamp_opt(9000, 0).unwrap(), Utc)
            )),
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            end_at: Some((
                Column::EndTime,
                &UtcTime::from_utc(NaiveDateTime::from_timestamp_opt(15000, 0).unwrap(), Utc)
            )),
            ..Default::default()
        }
        .execute(&pool)
        .await?
        .len(),
        2
    );

    assert_eq!(
        ListYouTubeStreamsQuery {
            order_by: Some((Column::ScheduleTime, Ordering::Asc)),
            ..Default::default()
        }
        .execute(&pool)
        .await?[0]
            .stream_id,
        2
    );

    Ok(())
}
