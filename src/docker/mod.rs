#![warn(clippy::all, clippy::pedantic)]

mod connection;
mod models;

use crate::error::WhaleError;
use connection::{Connection, ConnectionFactory};
pub use models::{Container, Port};

pub struct DockerClient {
    connection: Box<dyn Connection>,
}

impl DockerClient {
    pub async fn new() -> Result<Self, WhaleError> {
        let connection = ConnectionFactory::create()?;
        Ok(Self {
            connection: Box::new(connection),
        })
    }

    pub async fn list_containers(&self) -> Result<Vec<Container>, WhaleError> {
        self.connection.list_containers().await
    }
}
