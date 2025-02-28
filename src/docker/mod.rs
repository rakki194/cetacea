#![warn(clippy::all, clippy::pedantic)]

mod connection;
pub mod models;

use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::{
    client::legacy::{Client, Error as HyperUtilError},
    rt::TokioExecutor,
};
use log::{debug, error, trace};
use thiserror::Error;
pub use models::{Container, Port, ContainerStats};

#[derive(Error, Debug)]
pub enum WhaleError {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    #[error("HTTP client error: {0}")]
    HyperUtil(#[from] HyperUtilError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct DockerClient {
    client: Client<hyperlocal::UnixConnector, Empty<Bytes>>,
}

impl DockerClient {
    pub fn new() -> Self {
        debug!("Creating new DockerClient");
        Self {
            client: Client::builder(TokioExecutor::new()).build(hyperlocal::UnixConnector),
        }
    }

    pub async fn list_containers(&self) -> Result<Vec<Container>, WhaleError> {
        let uri = hyperlocal::Uri::new("/var/run/docker.sock", "/v1.43/containers/json?all=true")
            .into();

        debug!("Listing containers");
        let response = self.client.get(uri).await?;
        let body = response.into_body().collect().await?.to_bytes();
        trace!("List containers response: {}", String::from_utf8_lossy(&body));
        let containers: Vec<Container> = serde_json::from_slice(&body)?;
        debug!("Found {} containers", containers.len());
        Ok(containers)
    }

    pub fn list_containers_blocking(&self) -> Result<Vec<Container>, WhaleError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(self.list_containers())
    }

    pub async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStats, WhaleError> {
        let path = format!("/v1.43/containers/{container_id}/stats?stream=false");
        let uri = hyperlocal::Uri::new("/var/run/docker.sock", &path).into();

        debug!("Requesting stats for container: {}", container_id);
        let response = self.client.get(uri).await?;
        let body = response.into_body().collect().await?.to_bytes();
        trace!("Stats response for {}: {}", container_id, String::from_utf8_lossy(&body));
        
        match serde_json::from_slice::<ContainerStats>(&body) {
            Ok(stats) => {
                debug!("Successfully parsed stats for container {}", container_id);
                Ok(stats)
            }
            Err(e) => {
                error!("Failed to parse stats for container {}: {}", container_id, e);
                error!("Error location: line {}, column {}", e.line(), e.column());
                Err(WhaleError::Json(e))
            }
        }
    }

    pub fn get_container_stats_blocking(&self, container_id: &str) -> Result<ContainerStats, WhaleError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(self.get_container_stats(container_id))
    }
}
