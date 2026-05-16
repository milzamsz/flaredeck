use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("cloudflared not found in PATH or common install locations")]
    CloudflaredMissing,
    #[error("cloudflared exited with status {0}: {1}")]
    CloudflaredFailed(i32, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("dns error: {0}")]
    Dns(String),
    #[error("profile {0} not found")]
    ProfileNotFound(String),
    #[error("profile {0} is already running")]
    ProfileAlreadyRunning(String),
    #[error("home directory not resolvable")]
    NoHomeDir,
    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::Other(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        AppError::Other(value.to_string())
    }
}
