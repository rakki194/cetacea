use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use colored::*;

mod docker;
mod error;

use crate::docker::{DockerClient, Container, Port, Health};

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

fn format_container_status(container: &Container) -> ColoredString {
    match container.state.as_str() {
        "running" => {
            if let Some(health) = &container.health {
                match health.status.as_str() {
                    "healthy" => "●".green(),
                    "unhealthy" => {
                        let latest_log = health.log.last().map(|log| log.output.as_str()).unwrap_or("");
                        format!("● ({})", latest_log).red()
                    }
                    _ => "●".yellow()
                }
            } else {
                "●".green()
            }
        }
        _ => "●".red()
    }
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
            println!("\n{} {} ({})", 
                format_container_status(&container),
                container.names.join(", "), 
                container.id[..12].to_string()
            );
            println!(" {}", container.image);
            println!(" {}", container.command);
            println!(" {}", format_duration(container.created));
            println!("󰔵 {}", container.status);
            println!("󰈀 {}", format_ports(&container.ports));
            
            if let Some(health) = &container.health {
                if health.status == "unhealthy" {
                    if let Some(last_log) = health.log.last() {
                        println!(" Last health check failed: {}", last_log.output.red());
                    }
                }
            }
        }
    }

    if stopped.is_empty() {
        println!("No stopped containers");
    } else {
        for container in stopped {
            println!("\n{} {} ({})", 
                format_container_status(&container),
                container.names.join(", "), 
                container.id[..12].to_string()
            );
            println!(" {}", container.image);
            println!(" {}", container.command);
            println!(" {}", format_duration(container.created));
            println!("󰔵 {}", container.status);
            println!("󰈀 {}", format_ports(&container.ports));
        }
    }

    println!();
    Ok(())
}
