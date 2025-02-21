#![warn(clippy::all, clippy::pedantic)]

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::Request;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::rt::TokioExecutor;
use hyperlocal::UnixConnector;

use super::models::Container;
use crate::error::WhaleError;

#[async_trait]
pub trait Connection: Send + Sync {
    async fn list_containers(&self) -> Result<Vec<Container>, WhaleError>;
}

pub struct ConnectionFactory;

impl ConnectionFactory {
    pub fn create() -> Result<impl Connection, WhaleError> {
        #[cfg(target_os = "linux")]
        {
            Ok(UnixSocketConnection::new())
        }

        #[cfg(target_os = "windows")]
        {
            Ok(HttpConnection::new())
        }
    }
}

#[cfg(target_os = "linux")]
pub struct UnixSocketConnection {
    client: HyperClient<UnixConnector, http_body_util::Empty<Bytes>>,
}

#[cfg(target_os = "linux")]
impl UnixSocketConnection {
    pub fn new() -> Self {
        Self {
            client: HyperClient::builder(TokioExecutor::new()).build(UnixConnector),
        }
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl Connection for UnixSocketConnection {
    async fn list_containers(&self) -> Result<Vec<Container>, WhaleError> {
        let uri = hyperlocal::Uri::new(
            "/var/run/docker.sock",
            "/v1.41/containers/json?all=true&health=true",
        );

        let req = Request::builder()
            .uri(uri)
            .header("Host", "")
            .body(http_body_util::Empty::new())
            .map_err(|e| WhaleError::Connection(e.to_string()))?;

        let response = self
            .client
            .request(req)
            .await
            .map_err(|e| WhaleError::Connection(e.to_string()))?;

        let body = response
            .collect()
            .await
            .map_err(|e| WhaleError::Connection(e.to_string()))?
            .to_bytes();

        serde_json::from_slice(&body).map_err(|e| WhaleError::Serialization(e.to_string()))
    }
}
