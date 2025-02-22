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
#[allow(dead_code)]  // Used through trait objects
pub trait Connection: Send + Sync {
    async fn list_containers(&self) -> Result<Vec<Container>, WhaleError>;
}

#[allow(dead_code)]  // Used as the factory for creating connections
pub struct ConnectionFactory;

impl ConnectionFactory {
    #[allow(dead_code)]  // Used to create connections
    pub fn create() -> impl Connection {
        #[cfg(target_os = "linux")]
        {
            UnixSocketConnection::new()
        }

        #[cfg(target_os = "windows")]
        {
            HttpConnection::new()
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
