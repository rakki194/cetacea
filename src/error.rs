#![warn(clippy::all, clippy::pedantic)]

use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]  // Error variants might be used in the future
pub enum WhaleError {
    #[error("Docker connection error: {0}")]
    Connection(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}
