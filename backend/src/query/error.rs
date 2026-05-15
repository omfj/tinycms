use crate::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    Invalid(String),
}

impl From<QueryError> for Error {
    fn from(e: QueryError) -> Self {
        Error::BadRequest(e.to_string())
    }
}
