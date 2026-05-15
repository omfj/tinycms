use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Error::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            Error::Sqlx(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "not found".into()),
            Error::Sqlx(e) => {
                tracing::error!("db: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            Error::Internal(e) => {
                tracing::error!("internal: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
