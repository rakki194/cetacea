#![warn(clippy::all, clippy::pedantic)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Container {
    #[serde(alias = "Id", alias = "ID")]
    pub id: String,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(alias = "Image")]
    pub image: String,
    #[serde(alias = "Command")]
    pub command: String,
    #[serde(alias = "Created")]
    pub created: i64,
    #[serde(alias = "State")]
    pub state: String,
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(default)]
    pub ports: Vec<Port>,
    #[serde(alias = "Health")]
    pub health: Option<Health>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Port {
    #[serde(rename = "IP")]
    pub ip: Option<String>,
    #[serde(rename = "PrivatePort")]
    pub internal: u16,
    #[serde(rename = "PublicPort")]
    pub external: Option<u16>,
    #[serde(rename = "Type")]
    pub protocol: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Health {
    pub status: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct ContainerStats {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub read: String,
    #[serde(default)]
    pub preread: String,
    #[serde(default)]
    pub cpu_stats: CpuStats,
    #[serde(default)]
    pub memory_stats: MemoryStats,
    #[serde(default)]
    pub precpu_stats: CpuStats,
    #[serde(rename = "nvidia_stats", default)]
    pub gpu_stats: Option<GpuStats>,
    #[serde(default)]
    pub pids_stats: serde_json::Value,
    #[serde(default)]
    pub blkio_stats: serde_json::Value,
    #[serde(default)]
    pub num_procs: u64,
    #[serde(default)]
    pub storage_stats: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CpuStats {
    #[serde(default)]
    pub cpu_usage: CpuUsage,
    #[serde(default)]
    pub system_cpu_usage: Option<u64>,
    #[serde(default)]
    pub online_cpus: Option<u32>,
    #[serde(default)]
    pub throttling_data: ThrottlingData,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CpuUsage {
    #[serde(default)]
    pub total_usage: u64,
    #[serde(default)]
    pub usage_in_kernelmode: u64,
    #[serde(default)]
    pub usage_in_usermode: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ThrottlingData {
    #[serde(default)]
    pub periods: u64,
    #[serde(default)]
    pub throttled_periods: u64,
    #[serde(default)]
    pub throttled_time: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MemoryStats {
    pub usage: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct GpuStats {
    #[serde(default)]
    pub devices: Vec<GpuDevice>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct GpuDevice {
    #[serde(default)]
    pub memory_used: u64,
    #[serde(default)]
    pub memory_total: u64,
    #[serde(default)]
    pub utilization: u32,
}
