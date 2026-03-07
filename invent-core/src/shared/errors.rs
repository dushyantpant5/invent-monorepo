use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use db::DbError;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("forbidden: {0}")]
    Forbidden(&'static str),

    #[error("unauthorized")]
    Unauthorized,

    #[error("internal error")]
    Internal,
}

impl From<DbError> for AppError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::NotFound => AppError::NotFound,
            DbError::Conflict(msg) => AppError::Conflict(msg),
            DbError::ForeignKey(msg) => AppError::BadRequest(msg),
            DbError::InvalidInput(msg) => AppError::BadRequest(msg),
            DbError::SeaOrm(err) => {
                tracing::error!("unhandled db error: {:?}", err);
                AppError::Internal
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".into(),
            ),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
