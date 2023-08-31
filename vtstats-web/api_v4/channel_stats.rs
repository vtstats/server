use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use vtstats_database::{channel_stats as db, PgPool};
use warp::{http::StatusCode, reply::Response, Rejection, Reply};

use crate::reject::WarpError;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqQuery {
    channel_id: i32,
    #[serde(default, with = "ts_milliseconds_option")]
    start_at: Option<DateTime<Utc>>,
    #[serde(default, with = "ts_milliseconds_option")]
    end_at: Option<DateTime<Utc>>,
}

impl ReqQuery {
    fn invalid(&self) -> bool {
        let n = Utc::now();

        if matches!(self.end_at, Some(e) if (e - n).num_days() > 0) {
            return true;
        }

        if matches!(self.start_at, Some(s) if (n - s).num_days() > 365) {
            return true;
        }

        if matches!((self.start_at, self.end_at), (Some(s), Some(e)) if s >= e) {
            return true;
        }

        false
    }
}

pub async fn channel_subscriber_stats(
    query: ReqQuery,
    pool: PgPool,
) -> Result<Response, Rejection> {
    if query.invalid() {
        // TODO: better error json format
        return Ok(StatusCode::UNPROCESSABLE_ENTITY.into_response());
    }

    let res = db::channel_subscriber_stats(query.channel_id, query.start_at, query.end_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&res).into_response())
}

pub async fn channel_view_stats(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    if query.invalid() {
        return Ok(StatusCode::UNPROCESSABLE_ENTITY.into_response());
    }

    let res = db::channel_view_stats(query.channel_id, query.start_at, query.end_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&res).into_response())
}

pub async fn channel_revenue_stats(query: ReqQuery, pool: PgPool) -> Result<Response, Rejection> {
    if query.invalid() {
        return Ok(StatusCode::UNPROCESSABLE_ENTITY.into_response());
    }

    let res = db::channel_revenue_stats(query.channel_id, query.start_at, query.end_at, &pool)
        .await
        .map_err(Into::<WarpError>::into)?;

    Ok(warp::reply::json(&res).into_response())
}

#[test]
fn test_invalid_date_range() {
    use chrono::Duration;

    assert!(!ReqQuery {
        channel_id: 0,
        start_at: None,
        end_at: None,
    }
    .invalid());

    let now = Utc::now();

    assert!(ReqQuery {
        channel_id: 0,
        start_at: Some(now - Duration::days(370)),
        end_at: None
    }
    .invalid());

    assert!(!ReqQuery {
        channel_id: 0,
        start_at: Some(now - Duration::days(360)),
        end_at: None
    }
    .invalid());

    assert!(!ReqQuery {
        channel_id: 0,
        start_at: None,
        end_at: Some(now + Duration::hours(23)),
    }
    .invalid());

    assert!(ReqQuery {
        channel_id: 0,
        start_at: None,
        end_at: Some(now + Duration::hours(25)),
    }
    .invalid());

    assert!(ReqQuery {
        channel_id: 0,
        start_at: Some(now - Duration::days(100)),
        end_at: Some(now - Duration::days(200)),
    }
    .invalid());
}
