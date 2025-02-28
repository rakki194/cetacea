#![warn(clippy::all, clippy::pedantic)]

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::Request;
use hyper_util::client::legacy::Client as HyperClient;
use hyper_util::rt::TokioExecutor;
use hyperlocal::UnixConnector;
use log::{debug, error, trace};

use super::models::{Container, ContainerStats};
use crate::error::WhaleError;

#[async_trait]
#[allow(dead_code)]  // Used through trait objects
pub trait Connection: Send + Sync {
    async fn list_containers(&self) -> Result<Vec<Container>, WhaleError>;
    async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStats, WhaleError>;
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
        debug!("Creating new UnixSocketConnection");
        Self {
            client: HyperClient::builder(TokioExecutor::new()).build(UnixConnector),
        }
    }

    async fn make_request(&self, path: &str) -> Result<Bytes, WhaleError> {
        let uri = hyperlocal::Uri::new("/var/run/docker.sock", path);
        debug!("Making request to {}", path);

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

        trace!("Response status: {}", response.status());
        
        response
            .collect()
            .await
            .map_err(|e| WhaleError::Connection(e.to_string()))
            .map(|body| body.to_bytes())
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl Connection for UnixSocketConnection {
    async fn list_containers(&self) -> Result<Vec<Container>, WhaleError> {
        let body = self.make_request("/v1.43/containers/json?all=true&health=true").await?;
        trace!("List containers response: {}", String::from_utf8_lossy(&body));
        match serde_json::from_slice(&body) {
            Ok(containers) => {
                debug!("Successfully parsed container list");
                Ok(containers)
            }
            Err(e) => {
                error!("Failed to parse container list: {}", e);
                error!("Error location: line {}, column {}", e.line(), e.column());
                error!("Raw JSON response: {}", String::from_utf8_lossy(&body));
                Err(WhaleError::Serialization(e.to_string()))
            }
        }
    }

    async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStats, WhaleError> {
        let path = format!("/v1.43/containers/{container_id}/stats?stream=false");
        debug!("Requesting stats for container: {}", container_id);
        let body = self.make_request(&path).await?;
        trace!("Stats response for {}: {}", container_id, String::from_utf8_lossy(&body));
        match serde_json::from_slice::<ContainerStats>(&body) {
            Ok(stats) => {
                debug!("Successfully parsed stats for container {}", container_id);
                Ok(stats)
            }
            Err(e) => {
                error!("Failed to parse stats for container {}: {}", container_id, e);
                error!("Error location: line {}, column {}", e.line(), e.column());
                error!("Raw JSON response: {}", String::from_utf8_lossy(&body));
                Err(WhaleError::Serialization(e.to_string()))
            }
        }
    }
}
