use clap::{Arg, ArgMatches};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime};

#[derive(Debug, Serialize, Deserialize)]
struct ClusterInfo {
    servers: Vec<ServerInfo>,
    backend: BackendInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    endpoint: String,
    state: String,
    uptime: u64,
    version: String,
    disks: Vec<DiskInfo>,
    network: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiskInfo {
    state: String,
    available_space: u64,
    total_space: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BackendInfo {
    total_sets: Vec<i32>,
    drives_per_set: Vec<i32>,
    online_disks: i32,
    offline_disks: i32,
    standard_parity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClusterStruct {
    status: String,
    error: Option<String>,
    info: Option<ClusterInfo>,
    only_offline: bool,
}

impl ClusterStruct {
    fn new() -> ClusterStruct {
        ClusterStruct {
            status: "success".to_string(),
            error: None,
            info: None,
            only_offline: false,
        }
    }
}

impl fmt::Display for ClusterStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.status == "error" {
            writeln!(
                f,
                "Error: {}",
                self.error.as_ref().unwrap_or(&"Unknown error".to_string())
            )?;
            return Ok(());
        }

        let info = self.info.as_ref().ok_or(fmt::Error)?;

        for srv in &info.servers {
            if srv.state != "online" {
                writeln!(f, "{} {} (Offline)", "●".red(), srv.endpoint.blue())?;
                writeln!(f, "   Uptime: {}", format_duration(srv.uptime).red())?;
                writeln!(
                    f,
                    "   Drives: {} / {}",
                    srv.disks.iter().filter(|d| d.state == "ok").count(),
                    srv.disks.len()
                )?;
            } else if !self.only_offline {
                writeln!(f, "{} {} (Online)", "●".green(), srv.endpoint.blue())?;
                writeln!(f, "   Uptime: {}", format_duration(srv.uptime).green())?;
                writeln!(f, "   Version: {}", srv.version)?;
            }
        }

        writeln!(f, "Backend summary:")?;
        writeln!(f, "   Sets: {:?}", info.backend.total_sets)?;
        writeln!(f, "   Drives per set: {:?}", info.backend.drives_per_set)?;
        writeln!(f, "   Online drives: {}", info.backend.online_disks)?;
        writeln!(f, "   Offline drives: {}", info.backend.offline_disks)?;
        writeln!(
            f,
            "   Erasure Code Parity: {}",
            info.backend.standard_parity
        )?;

        Ok(())
    }
}

fn format_duration(secs: u64) -> String {
    let duration = Duration::from_secs(secs);
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    format!("{}h {}m", hours, minutes)
}

fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let target = matches.value_of("TARGET").ok_or("TARGET is required")?;
    let only_offline = matches.is_present("offline");

    // Simulate a client request and response
    let mut cluster_info = ClusterStruct::new();
    cluster_info.only_offline = only_offline;

    // Here you'd connect to MinIO and retrieve the data.
    let servers = vec![
        ServerInfo {
            endpoint: format!("{}-server-1", target),
            state: "online".to_string(),
            uptime: 3600,
            version: "1.0.0".to_string(),
            disks: vec![
                DiskInfo {
                    state: "ok".to_string(),
                    available_space: 10_000,
                    total_space: 20_000,
                },
                DiskInfo {
                    state: "failed".to_string(),
                    available_space: 0,
                    total_space: 20_000,
                },
            ],
            network: vec!["online".to_string()],
        },
        ServerInfo {
            endpoint: format!("{}-server-2", target),
            state: "offline".to_string(),
            uptime: 0,
            version: "1.0.0".to_string(),
            disks: vec![DiskInfo {
                state: "failed".to_string(),
                available_space: 0,
                total_space: 20_000,
            }],
            network: vec!["offline".to_string()],
        },
    ];

    let backend_info = BackendInfo {
        total_sets: vec![1],
        drives_per_set: vec![2],
        online_disks: 1,
        offline_disks: 1,
        standard_parity: 1,
    };

    cluster_info.info = Some(ClusterInfo {
        servers,
        backend: backend_info,
    });

    // Print cluster information
    println!("{}", cluster_info);

    Ok(())
}
