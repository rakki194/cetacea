#![warn(clippy::all, clippy::pedantic)]

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

mod docker;
mod error;
mod tui;

use crate::docker::{Container, DockerClient};
use crate::tui::App;

fn format_duration(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX);
    let duration = now - timestamp;

    if duration < 60 {
        format!("{duration} seconds ago")
    } else if duration < 3600 {
        format!("{} minutes ago", duration / 60)
    } else if duration < 86400 {
        format!("{} hours ago", duration / 3600)
    } else {
        format!("{} days ago", duration / 86400)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = DockerClient::new();
    let mut containers = client.list_containers().await?;
    
    // Sort containers: running first, then by name
    containers.sort_by(|a, b| {
        let state_order = b.state.cmp(&a.state);
        if state_order == std::cmp::Ordering::Equal {
            a.names[0].cmp(&b.names[0])
        } else {
            state_order
        }
    });

    let mut app = App::new(containers, client);
    app.run()?;

    Ok(())
}
