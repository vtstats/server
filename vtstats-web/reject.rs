use anyhow::Error as AnyhowError;
use std::convert::Infallible;
use std::error::Error;
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
    pub message: &'static str,
}

pub async fn handle_rejection(err: Rejection) -> Result<Response, Infallible> {
    if let Some(i) = err.find::<NeedLogin>() {
        return Ok(i.into_response());
    }

    let (code, message) = get_response(err);

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code).into_response())
}

fn get_response(err: Rejection) -> (StatusCode, &'static str) {
    if err.is_not_found() {
        (StatusCode::NOT_FOUND, "NOT_FOUND")
    } else if let Some(WarpError(err)) = err.find::<WarpError>() {
        tracing::error!(exception.stacktrace = ?err, message= %err);
        (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR")
    } else if err.find::<warp::reject::InvalidQuery>().is_some() {
        (StatusCode::UNPROCESSABLE_ENTITY, "INVALID_QUERY")
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (StatusCode::METHOD_NOT_ALLOWED, "METHOD_NOT_ALLOWED")
    } else {
        tracing::error!(message= ?err);
        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION")
    }
}
