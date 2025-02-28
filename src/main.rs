#![warn(clippy::all, clippy::pedantic)]

use std::error::Error;
use clap::Parser;
use log::{info, warn};

mod docker;
mod error;
mod tui;
mod utils;
#[cfg(test)]
mod tests;

use crate::docker::DockerClient;
use crate::tui::App;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Refresh rate in milliseconds
    #[arg(short, long, default_value_t = 250)]
    refresh_rate: u64,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(args.log_level.parse().unwrap_or(log::LevelFilter::Info))
        .init();

    info!("Starting cetacea with refresh rate: {}ms", args.refresh_rate);
    
    let client = DockerClient::new();
    let mut containers = client.list_containers().await?;
    
    // Sort containers: running first, then by name
    containers.sort_by(|a, b| {
        let state_order = b.state.cmp(&a.state);
        if state_order == std::cmp::Ordering::Equal {
            let a_name = a.names.first().map_or("", |s| s.as_str());
            let b_name = b.names.first().map_or("", |s| s.as_str());
            a_name.cmp(b_name)
        } else {
            state_order
        }
    });

    info!("Found {} containers", containers.len());
    
    let app = App::new(containers, client);
    app.run_with_options(std::time::Duration::from_millis(args.refresh_rate))?;

    Ok(())
}
