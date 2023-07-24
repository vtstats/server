use anyhow::Error as AnyhowError;
use std::convert::Infallible;
use std::error::Error;
use tracing::field::{debug, display};
use tracing::Span;
use vtstat_database::DatabaseError;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::reply::Response;
use warp::{Rejection, Reply};

use integration_googleauth::validate::NeedLogin;

#[derive(Debug)]
pub struct WarpError(pub AnyhowError);

impl Reject for WarpError {}

impl<T: Error + Send + Sync + 'static> From<T> for WarpError {
    fn from(err: T) -> Self {
        WarpError(AnyhowError::new(err))
    }
}

#[derive(serde::Serialize)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String,
}

pub async fn handle_rejection(err: Rejection) -> Result<Response, Infallible> {
    let code;
    let message;
    let description;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else {
        if let Some(i) = err.find::<NeedLogin>() {
            return Ok(i.into_response());
        }
        if let Some(WarpError(err)) = err.find::<WarpError>() {
            Span::current()
                .record("error.message", display(err))
                .record("error.cause_chain", debug(err));

            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "INTERNAL_SERVER_ERROR";
            description = if err.is::<DatabaseError>() {
                "DATABASE_ERROR"
            } else {
                "INTERNAL_SERVER_ERROR"
            }
        } else if err.find::<warp::reject::InvalidQuery>().is_some() {
            code = StatusCode::UNPROCESSABLE_ENTITY;
            message = "INVALID_QUERY";
            description = "INVALID_QUERY";
        } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
            code = StatusCode::METHOD_NOT_ALLOWED;
            message = "METHOD_NOT_ALLOWED";
            description = "METHOD_NOT_ALLOWED";
        } else {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "UNHANDLED_REJECTION";
            description = "UNHANDLED_REJECTION";
        }

        Span::current()
            .record("otel.status_code", "ERROR")
            .record("otel.status_description", description);
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code).into_response())
}
