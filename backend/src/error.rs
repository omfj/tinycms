use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::schema::FieldError;

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

    #[error("validation failed")]
    Validation(Vec<FieldError>),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": errors })),
            )
                .into_response(),
            ref e => {
                let (status, msg) = match e {
                    Error::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
                    Error::Unauthorized => (StatusCode::UNAUTHORIZED, e.to_string()),
                    Error::Forbidden => (StatusCode::FORBIDDEN, e.to_string()),
                    Error::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
                    Error::Sqlx(sqlx::Error::RowNotFound) => {
                        (StatusCode::NOT_FOUND, "not found".into())
                    }
                    Error::Sqlx(err) => {
                        tracing::error!("db: {err}");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "internal server error".into(),
                        )
                    }
                    Error::Internal(err) => {
                        tracing::error!("internal: {err}");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "internal server error".into(),
                        )
                    }
                    Error::Validation(_) => unreachable!(),
                };
                (status, Json(json!({ "error": msg }))).into_response()
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
