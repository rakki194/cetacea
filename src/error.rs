#![warn(clippy::all, clippy::pedantic)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WhaleError {
    #[error("Docker connection error: {0}")]
    Connection(String),

    #[error("Docker API error: {0}")]
    Api(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
