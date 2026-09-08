use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Device identity error: {0}")]
    DeviceError(String),

    #[error("Database persistence error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
