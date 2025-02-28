use std::io;
use thiserror::Error;

pub mod docker;
pub mod error;
pub mod tui;
pub mod utils;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Terminal error: {0}")]
    Terminal(#[from] ratui_lib::Error),
    #[error("Docker error: {0}")]
    Docker(String),
    #[error("Whale error: {0}")]
    Whale(#[from] error::WhaleError),
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, err.to_string())
    }
} 