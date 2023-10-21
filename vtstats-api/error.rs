use anyhow::Error as AnyhowError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct ApiError(pub AnyhowError);

impl<T: Into<AnyhowError>> From<T> for ApiError {
    fn from(err: T) -> Self {
        ApiError(err.into())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("Error: {}", self.0);
        tracing::error!("Stack: {:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
