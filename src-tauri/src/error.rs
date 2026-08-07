use serde::Serialize;

#[derive(Debug, Clone, Serialize, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Database(String),
}

impl From<libsql::Error> for AppError {
    fn from(e: libsql::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<serde::de::value::Error> for AppError {
    fn from(e: serde::de::value::Error) -> Self {
        AppError::Database(format!("Décodage de ligne SQL : {e}"))
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Validation(s)
    }
}
