use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

mod docker;
mod error;

use crate::docker::{DockerClient, Container, Port};

fn format_duration(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let duration = now - timestamp;

    if duration < 60 {
        format!("{} seconds ago", duration)
    } else if duration < 3600 {
        format!("{} minutes ago", duration / 60)
    } else if duration < 86400 {
        format!("{} hours ago", duration / 3600)
    } else {
        format!("{} days ago", duration / 86400)
    }
}

fn format_ports(ports: &[Port]) -> String {
    if ports.is_empty() {
        return "None".to_string();
    }

    ports
        .iter()
        .map(|p| {
            let public = p.public_port.map_or(String::new(), |port| format!("{}:", port));
            let ip = p.ip.as_deref().unwrap_or("");
            format!("{}{}:{}/{}", ip, public, p.private_port, p.protocol.to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = DockerClient::new().await?;
    let containers = client.list_containers().await?;

    let (running, stopped): (Vec<_>, Vec<_>) = containers
        .into_iter()
        .partition(|c| c.state == "running");

    if running.is_empty() {
        println!("No running containers");
    } else {
        for container in running {
            println!("\n{} ({})", container.names.join(", "), container.id[..12].to_string());
            println!(" {}", container.image);
            println!(" {}", container.command);
            println!(" {}", format_duration(container.created));
            println!(" {}", container.status);
            println!("󰈀 {}", format_ports(&container.ports));
        }
    }

    if stopped.is_empty() {
        println!("No stopped containers");
    } else {
        for container in stopped {
            println!("\n{} ({})", container.names.join(", "), container.id[..12].to_string());
            println!(" {}", container.image);
            println!(" {}", container.command);
            println!(" {}", format_duration(container.created));
            println!(" {}", container.status);
            println!("󰈀 {}", format_ports(&container.ports));
        }
    }

    println!();
    Ok(())
}
