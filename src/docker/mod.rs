#![warn(clippy::all, clippy::pedantic)]

mod connection;
mod models;

use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_util::{
    client::legacy::{Client, Error as HyperUtilError},
    rt::TokioExecutor,
};
use thiserror::Error;
pub use models::{Container, Port};

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
        Self {
            client: Client::builder(TokioExecutor::new()).build(hyperlocal::UnixConnector::default()),
        }
    }

    pub async fn list_containers(&self) -> Result<Vec<Container>, WhaleError> {
        let uri = hyperlocal::Uri::new("/var/run/docker.sock", "/v1.43/containers/json?all=true")
            .into();

        let response = self.client.get(uri).await?;
        let body = response.into_body().collect().await?.to_bytes();
        let containers: Vec<Container> = serde_json::from_slice(&body)?;
        Ok(containers)
    }

    pub fn list_containers_blocking(&self) -> Result<Vec<Container>, WhaleError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(self.list_containers())
    }
}
