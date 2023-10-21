use axum::{
    extract::{Query, State},
    http::status::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{serde::ts_milliseconds_option, DateTime, Utc};
use vtstats_database::{channel_stats as db, PgPool};

use crate::error::ApiResult;

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
    fn invalid_response(&self) -> Option<Response> {
        let n = Utc::now();

        if matches!(self.end_at, Some(e) if (e - n).num_days() > 0) {
            return Some(StatusCode::UNPROCESSABLE_ENTITY.into_response());
        }

        if matches!(self.start_at, Some(s) if (n - s).num_days() > 365) {
            return Some(StatusCode::UNPROCESSABLE_ENTITY.into_response());
        }

        if matches!((self.start_at, self.end_at), (Some(s), Some(e)) if s >= e) {
            return Some(StatusCode::UNPROCESSABLE_ENTITY.into_response());
        }

        None
    }
}

pub async fn channel_subscriber_stats(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<Response> {
    if let Some(res) = query.invalid_response() {
        return Ok(res);
    }

    let res =
        db::channel_subscriber_stats(query.channel_id, query.start_at, query.end_at, &pool).await?;

    Ok(Json(res).into_response())
}

pub async fn channel_view_stats(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<Response> {
    if let Some(res) = query.invalid_response() {
        return Ok(res);
    }

    let res = db::channel_view_stats(query.channel_id, query.start_at, query.end_at, &pool).await?;

    Ok(Json(res).into_response())
}

pub async fn channel_revenue_stats(
    Query(query): Query<ReqQuery>,
    State(pool): State<PgPool>,
) -> ApiResult<Response> {
    if let Some(res) = query.invalid_response() {
        return Ok(res);
    }

    let res =
        db::channel_revenue_stats(query.channel_id, query.start_at, query.end_at, &pool).await?;

    Ok(Json(res).into_response())
}

#[test]
fn test_invalid_date_range() {
    use chrono::Duration;

    assert!(ReqQuery {
        channel_id: 0,
        start_at: None,
        end_at: None,
    }
    .invalid_response()
    .is_none());

    let now = Utc::now();

    assert!(ReqQuery {
        channel_id: 0,
        start_at: Some(now - Duration::days(370)),
        end_at: None
    }
    .invalid_response()
    .is_some());

    assert!(ReqQuery {
        channel_id: 0,
        start_at: Some(now - Duration::days(360)),
        end_at: None
    }
    .invalid_response()
    .is_none());

    assert!(ReqQuery {
        channel_id: 0,
        start_at: None,
        end_at: Some(now + Duration::hours(23)),
    }
    .invalid_response()
    .is_none());

    assert!(ReqQuery {
        channel_id: 0,
        start_at: None,
        end_at: Some(now + Duration::hours(25)),
    }
    .invalid_response()
    .is_some());

    assert!(ReqQuery {
        channel_id: 0,
        start_at: Some(now - Duration::days(100)),
        end_at: Some(now - Duration::days(200)),
    }
    .invalid_response()
    .is_some());
}
