use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Vault error: {0}")]
    Vault(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Dialog error: {0}")]
    Dialog(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<ssh2::Error> for AppError {
    fn from(err: ssh2::Error) -> Self {
        AppError::Ssh(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
