use thiserror::Error;

#[derive(Error, Debug)]
pub enum WhaleError {
    #[error("Docker connection error: {0}")]
    ConnectionError(String),
    
    #[error("Docker API error: {0}")]
    ApiError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
