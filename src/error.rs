#![warn(clippy::all, clippy::pedantic)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WhaleError {
    #[error("Docker connection error: {0}")]
    Connection(String),

    #[error("Docker API error: {0}")]
    #[allow(dead_code)]
    Api(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
