#![warn(clippy::all, clippy::pedantic)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Container {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Names")]
    pub names: Vec<String>,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "ImageID")]
    pub image_id: String,
    #[serde(rename = "Command")]
    pub command: String,
    #[serde(rename = "Created")]
    pub created: i64,
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Health")]
    pub health: Option<Health>,
    #[serde(rename = "Ports")]
    pub ports: Vec<Port>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    #[serde(rename = "IP")]
    pub ip: Option<String>,
    #[serde(rename = "PrivatePort")]
    pub private_port: u16,
    #[serde(rename = "PublicPort")]
    pub public_port: Option<u16>,
    #[serde(rename = "Type")]
    pub protocol: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Health {
    pub status: String,
    #[serde(rename = "FailingStreak")]
    pub failing_streak: i32,
    pub log: Vec<HealthLog>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HealthLog {
    pub start: String,
    pub end: String,
    #[serde(rename = "ExitCode")]
    pub exit_code: i32,
    pub output: String,
}