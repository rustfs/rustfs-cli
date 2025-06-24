#![allow(unused_variables)]
#![allow(unused_imports)]
use chrono::{DateTime, Utc};
use colored::Colorize;
use human_bytes::human_bytes;
use humantime;
use pluralizer::pluralize;
use prettytable::{format, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum BackendType {
    Unknown = 0, // 与 iota 初始值相同
    FS,          // Filesystem backend
    Erasure,     // Multi disk Erasure (single, distributed) backend
    Gateway,     // Gateway to other storage
                 // Add your own backend types here as needed
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Services {}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoMessage {
    //#[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domain: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "sqsARN")]
    pub sqs_arn: Vec<String>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deploymentID"
    )]
    pub deployment_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buckets: Option<Buckets>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objects: Option<Objects>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versions: Option<Versions>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deletemarkers"
    )]
    pub delete_markers: Option<DeleteMarkers>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub services: Option<Services>,

    //#[serde(default, skip_serializing_if = "Option::is_none")]
    //#[serde(flatten)]
    pub backend: Option<ErasureBackend>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<ServerProperties>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub pools: HashMap<i32, HashMap<i32, ErasureSetInfo>>,
}

impl InfoMessage {
    // Equivalent to the BackendType() method in Go
    pub fn backend_type(&self) -> BackendType {
        //match &self.backend {
        // Some(backend) => match backend.as_str() {
        //     "Erasure" => BackendType::Erasure,
        //     "FS" => BackendType::FS,
        //     _ => BackendType::Unknown,
        // },

        // None => BackendType::Unknown,
        BackendType::Erasure
    }

    // Equivalent to the StandardParity() method in Go
    pub fn standard_parity(&self) -> i32 {
        match self.backend_type() {
            BackendType::Erasure => self
                .backend
                .as_ref()
                .and_then(|b| Some(b.standard_sc_parity))
                .unwrap_or(-1),
            _ => -1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ItemState {
    Offline,
    Initializing,
    Online,
}

impl fmt::Display for ItemState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match self {
            ItemState::Offline => "offline",
            ItemState::Initializing => "initializing",
            ItemState::Online => "online",
        };
        write!(f, "{}", state_str)
    }
}

impl ItemState {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "offline" => Some(ItemState::Offline),
            "initializing" => Some(ItemState::Initializing),
            "online" => Some(ItemState::Online),
            _ => None,
        }
    }
}

use std::time::Duration;
#[derive(Debug, Clone)]
pub struct TimedAction {
    pub count: u64,
    pub acc_time_ns: u64,
    pub bytes: u64,
}

impl TimedAction {
    // 返回平均时间
    pub fn avg(&self) -> Duration {
        if self.count == 0 {
            Duration::new(0, 0)
        } else {
            Duration::from_nanos(self.acc_time_ns / self.count)
        }
    }

    // 返回平均字节数
    pub fn avg_bytes(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.bytes / self.count
        }
    }

    // 合并另一个 TimedAction
    pub fn merge(&mut self, other: &TimedAction) {
        self.count += other.count;
        self.acc_time_ns += other.acc_time_ns;
        self.bytes += other.bytes;
    }
}

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DiskMetrics {
    #[serde(skip)]
    pub last_minute: HashMap<String, TimedAction>,
    #[serde(skip)]
    pub api_calls: HashMap<String, u64>,

    pub total_tokens: Option<u32>,
    pub total_waiting: Option<u32>,

    pub total_errors_availability: Option<u64>,
    pub total_errors_timeout: Option<u64>,

    pub total_writes: Option<u64>,
    pub total_deletes: Option<u64>,

    pub api_latencies: Option<HashMap<String, serde_json::Value>>,
}

use crate::clientadmin;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HealingDisk {
    pub id: String,
    pub heal_id: String,
    pub pool_index: i32,
    pub set_index: i32,
    pub disk_index: i32,
    pub endpoint: String,
    pub path: String,
    pub started: DateTime<Utc>,
    pub last_update: DateTime<Utc>,

    pub retry_attempts: u64,

    pub objects_total_count: u64,
    pub objects_total_size: u64,

    pub items_healed: u64,
    pub items_failed: u64,
    pub items_skipped: u64,
    pub bytes_done: u64,
    pub bytes_failed: u64,
    pub bytes_skipped: u64,

    pub objects_healed: Option<u64>, // Deprecated, kept as Option for compatibility
    pub objects_failed: Option<u64>, // Deprecated, kept as Option for compatibility

    pub bucket: String,
    pub object: String,

    pub queued_buckets: Vec<String>,
    pub healed_buckets: Vec<String>,

    pub finished: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Disk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_disk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drive_path: Option<String>,
    pub healing: Option<bool>,
    pub scanning: Option<bool>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub major: u32,
    pub minor: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "totalspace")]
    pub total_space: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "usedspace")]
    pub used_space: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "availspace")]
    pub available_space: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_throughput: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_throughput: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_latency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_latency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<DiskMetrics>,
    pub heal_info: Option<HealingDisk>,
    pub used_inodes: u64,
    pub free_inodes: Option<u64>,
    pub local: Option<bool>,

    pub pool_index: i32,
    pub set_index: i32,
    pub disk_index: i32,
}

pub type BackendDisks = HashMap<String, i32>;

pub trait BackendDiskOps {
    fn sum(&self) -> i32;
    fn merge(&self, other: &BackendDisks) -> BackendDisks;
}
impl BackendDiskOps for BackendDisks {
    // Sum - Returns the sum of the disk values in the map.
    fn sum(&self) -> i32 {
        self.values().sum()
    }

    // Merge - Reduces two endpoint-disk maps into a single map.
    fn merge(&self, other: &BackendDisks) -> BackendDisks {
        let mut merged = self.clone();

        for (key, &value) in other {
            *merged.entry(key.clone()).or_insert(0) += value;
        }

        merged
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackendInfo {
    // Represents various backend types, currently on FS, Erasure, and Gateway
    #[serde(skip, rename = "backendType")]
    pub backend_type: String,

    // Following fields are only meaningful if BackendType is Gateway.
    #[serde(rename = "gatewayOnlin")]
    pub gateway_online: bool,

    // Following fields are only meaningful if BackendType is Erasure.
    #[serde(rename = "onlineDisks")]
    pub online_disks: BackendDisks, // Online disks during server startup.
    #[serde(rename = "offlineDisks")]
    pub offline_disks: BackendDisks, // Offline disks during server startup.

    // Data and parity disks for Standard storage class configuration
    #[serde(rename = "standardScData")]
    pub standard_sc_data: Vec<i32>,
    pub standard_sc_parities: Vec<i32>,

    // Data and parity disks for Reduced Redundancy storage class configuration
    pub rrsc_data: Vec<i32>,
    pub rrsc_parities: Vec<i32>,

    // Number of erasure sets and drives per set per pool
    #[serde(rename = "totalSets")]
    pub total_sets: Vec<i32>,
    #[serde(rename = "totalDrivesPerSet")]
    pub drives_per_set: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageInfo {
    // List of disks
    pub disks: Vec<Disk>,

    // Backend information
    #[serde(rename = "backend")]
    pub backend: BackendInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketUsageInfo {
    pub size: u64,
    pub replication_pending_size: u64,
    pub replication_failed_size: u64,
    pub replicated_size: u64,
    pub replica_size: u64,
    pub replication_pending_count: u64,
    pub replication_failed_count: u64,
    pub versions_count: u64,
    pub objects_count: u64,
    pub delete_markers_count: u64,
    pub object_sizes_histogram: HashMap<String, u64>,
    pub object_versions_histogram: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct DataUsageInfo {
    pub last_update: DateTime<Utc>,
    pub objects_total_count: u64,
    pub objects_total_size: u64,
    pub replication_pending_size: u64,
    pub replication_failed_size: u64,
    pub replicated_size: u64,
    pub replica_size: u64,
    pub replication_pending_count: u64,
    pub replication_failed_count: u64,
    pub buckets_count: u64,
    pub buckets_usage: HashMap<String, BucketUsageInfo>,
    pub tier_stats: HashMap<String, TierStats>,
    pub bucket_sizes: HashMap<String, u64>, // Deprecated, kept for backward compatibility
    pub total_capacity: u64,
    pub total_free_capacity: u64,
    pub total_used_capacity: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErasureSetInfo {
    pub id: Option<i32>, // 使用 Option<i32> 代替 int
    #[serde(rename = "rawUsage")]
    pub raw_usage: u64, // 代表原始使用量
    #[serde(rename = "rawCapacity")]
    pub raw_capacity: u64, // 代表原始容量
    pub usage: Option<u64>, // 代表使用量
    #[serde(rename = "objectsCount")]
    pub objects_count: u64, // 代表对象计数
    #[serde(rename = "versionsCount")]
    pub versions_count: u64, // 代表版本计数
    #[serde(rename = "deleteMarkersCount")]
    pub delete_markers_count: u64, // 代表删除标记计数
    #[serde(rename = "healDisks")]
    pub heal_disks: i32, // 代表需要修复的磁盘数量
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerProperties {
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub state: String, // 状态
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: String, // 端点
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>, // 方案
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: i64, // 运行时间
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub version: String, // 版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>, // 提交 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<HashMap<String, String>>, // 网络信息
    #[serde(skip_serializing_if = "Option::is_none", rename = "drives")]
    pub disks: Option<Vec<Disk>>, // 磁盘信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_number: Option<i32>, // 池数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_numbers: Option<Vec<i32>>, // 池编号
    #[serde(default, skip)]
    pub mem_stats: MemStats, // 内存统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go_max_procs: Option<i32>, // Go 最大处理器数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_cpu: Option<i32>, // CPU 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>, // 运行时版本
    #[serde(default, skip)]
    pub gc_stats: Option<GCStats>, // 垃圾收集统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minio_env_vars: Option<HashMap<String, String>>, // MinIO 环境变量
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Buckets {
    pub count: u64, // 桶的数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>, // 错误信息
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Objects {
    pub count: u64, // 对象的数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>, // 错误信息
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Versions {
    pub count: u64, // 版本的数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>, // 错误信息
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteMarkers {
    pub count: u64, // 删除标记的数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>, // 错误信息
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
    pub size: u64, // 使用的总大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>, // 错误信息
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TierStats {
    pub total_size: u64,   // 总大小
    pub num_versions: i32, // 版本数量
    pub num_objects: i32,  // 对象数量
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KMS {
    pub status: Option<String>,   // KMS 状态
    pub encrypt: Option<String>,  // 加密信息
    pub decrypt: Option<String>,  // 解密信息
    pub endpoint: Option<String>, // 端点信息
    pub version: Option<String>,  // 版本信息
}

#[derive(Debug, Clone)]
pub struct LDAP {
    pub status: Option<String>, // LDAP 状态
}

#[derive(Debug, Clone)]
pub struct Status {
    pub status: Option<String>, // 状态信息
}

// pub type Audit = std::collections::HashMap<String, Status>; // 审计日志状态

// pub type Logger = std::collections::HashMap<String, Status>; // 日志状态

// pub type TargetIDStatus = std::collections::HashMap<String, Status>; // 目标 ID 状态

#[derive(Debug, Clone)]
pub struct backendType(String); // 后端类型

impl backendType {
    pub const FS_TYPE: &'static str = "FS"; // 后端为 FS 类型
    pub const ERASURE_TYPE: &'static str = "Erasure"; // 后端为 Erasure 类型
    pub fn new(value: &str) -> Self {
        backendType(value.to_string())
    }

    // Optional: A method to access the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct FSBackend {
    pub r#type: BackendType, // 后端类型
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErasureBackend {
    pub backendType: BackendType, // 后端类型
    #[serde(rename = "onlineDisks")]
    pub online_disks: i32, // 在线磁盘数量
    #[serde(rename = "offlineDisks")]
    pub offline_disks: i32, // 离线磁盘数量
    #[serde(rename = "standardSCParity")]
    pub standard_sc_parity: i32, // 标准存储类的奇偶校验磁盘
    #[serde(rename = "rrSCParity")]
    pub rr_sc_parity: i32, // 减少冗余存储类的奇偶校验磁盘
    #[serde(rename = "totalSets")]
    pub total_sets: Vec<i32>, // 每个池的信息
    #[serde(rename = "totalDrivesPerSet")]
    pub drives_per_set: Vec<i32>, // 每个池的驱动器数量
}

#[derive(Debug, Default, Clone)]
pub struct ServerInfoOpts {
    pub metrics: bool, // 是否请求每个驱动器的额外指标
}

// WithDriveMetrics 函数用于设置 ServerInfoOpts 的指标选项
pub fn with_drive_metrics(metrics: bool) -> impl Fn(&mut ServerInfoOpts) {
    move |opts: &mut ServerInfoOpts| {
        opts.metrics = metrics; // 设置指标选项
    }
}

fn display_info(info: &InfoMessage) {
    // Display server endpoint and status
    for server in &info.servers {
        println!("• {}: {}", server.endpoint, server.state);
        //println!("  Uptime: {} minutes", server.uptime / 60);
        println!("  Version: {}", server.version);
        //println!("  Network: {}/1 OK", info.backend.unwrap().online_disks);
        //println!("  Drives: {}/{} OK", server.drives.len(), info.backend.unwrap().online_disks);
        //println!("  Pool: {}", server.pool_number);
    }

    // Display pool information in a table format
    println!();
    println!(
        "{:<10} {:<15} {:<20} {:<15}",
        "Pool", "Drives Usage", "Erasure stripe size", "Erasure sets"
    );
    for (pool_key, pool_data) in &info.pools {
        for (set_key, pool_info) in pool_data {
            let usage_percent =
                (pool_info.raw_usage as f64 / pool_info.raw_capacity as f64) * 100.0;
            println!(
                "{:<10} {:.1}% (total: {:.2} GiB) {:<20} {:<15}",
                pool_key,
                usage_percent,
                pool_info.raw_capacity as f64 / (1024.0 * 1024.0 * 1024.0),
                info.backend.as_ref().unwrap().total_sets[0],
                info.backend.as_ref().unwrap().total_sets.len(),
            );
        }
    }

    // Display summary
    // println!("\n{} MiB Used, {} Bucket, {} Objects", info.usage.size / (1024 * 1024), info.buckets.count, info.objects.count);
    // println!("{} drive online, {} drives offline, EC:{}", info.backend.onlineDisks, info.backend.offlineDisks, 0);
}

pub async fn ServerInfo() {
    let res = clientadmin::get_request(
        "12345678".to_string(),
        "12345678".to_string(),
        "http://127.0.0.1:9000/minio/admin/v3/info?metrics=false".to_string(),
        "us-east-1".to_string(),
    )
    .await;
    println!("\n");
    match res {
        Ok(response) => {
            let ret: Result<InfoMessage, serde_json::Error> = serde_json::from_str(&response);

            match ret {
                Ok(msg) => {
                    //println!("{}", response);
                    let x = ClusterStruct {
                        only_offline: false,
                        info: Some(msg),
                        status: "online".to_string(),
                        error: None,
                    };
                    println!("{}", x);
                }
                Err(e) => {
                    println!("json err: {}", e);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MemStats {
    pub alloc: u64,       // 当前分配的字节数
    pub total_alloc: u64, // 总分配的字节数
    pub mallocs: u64,     // 分配次数
    pub frees: u64,       // 释放次数
    pub heap_alloc: u64,  // 堆分配的字节数
}

// GCStats 结构体用于收集最近的垃圾回收信息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GCStats {
    pub last_gc: DateTime<Utc>,        // 上次回收的时间
    pub num_gc: i64,                   // 垃圾回收的次数
    pub pause_total: Duration,         // 所有回收的总暂停时间
    pub pause: Vec<Duration>,          // 暂停历史，最新的在前
    pub pause_end: Vec<DateTime<Utc>>, // 暂停结束时间历史，最新的在前
}

fn endpoint_to_pools(endpoint: &str, c: &ClusterInfo) -> Vec<i32> {
    let mut pools = Vec::new();

    for (&pool_number, pool_summary) in c {
        if pool_summary.endpoints.contains(endpoint) {
            pools.push(pool_number);
        }
    }

    pools.sort();
    pools
}

#[derive(Debug, Deserialize)]
pub struct ClusterStruct {
    status: String,
    error: Option<String>,
    info: Option<InfoMessage>,
    only_offline: bool,
}

impl fmt::Display for ClusterStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 检查集群状态中的错误
        if self.status == "error" {
            return write!(
                f,
                "Unable to get service info: {}",
                self.error.as_deref().unwrap_or("Unknown error")
            );
        }

        // 确保收集到信息
        let info = match &self.info {
            Some(info) => info,
            None => return write!(f, "Unable to get service info"),
        };

        // 验证服务器是否符合要求
        // if info.backend.unwrap().total_sets.is_empty() {
        //     return write!(f, "Unable to display service info, server is too old");
        // }
        // if info.backend.unwrap().total_sets.is_empty() || infounwrap()..backend.drives_per_set.is_empty() {
        //     return write!(f, "Unable to display service info, server is too old");
        // }

        // 初始化变量
        let mut total_offline_nodes = 0;
        let backend_type = info.backend_type();
        let colored_dot = if info.mode == "initializing".to_string() {
            "●".yellow().bold()
        } else {
            "●".green().bold()
        };

        // 按端点排序服务器
        let mut servers = info.servers.clone();
        servers.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));

        let cluster_summary = cluster_summary_info(info);

        // 输出表格
        //println!("{}", builder);

        // 遍历每个服务器并汇总信息
        for srv in &servers {
            if srv.state != "online" {
                total_offline_nodes += 1;
                writeln!(f, "{}  {}", "●".red().bold(), srv.endpoint.blue())?;
                writeln!(f, "   Uptime: {}", srv.state.red())?;

                if backend_type == BackendType::Erasure {
                    let (mut on_drives, mut off_drives) = (0, 0);
                    if let Some(drives) = &srv.disks {
                        for disk in drives {
                            if disk.state == "ok" || disk.state == "unformatted" {
                                on_drives += 1;
                            } else {
                                off_drives += 1;
                            }
                        }
                    }

                    let disp_no_of_drives = format!("{}/{}", on_drives, on_drives + off_drives);
                    writeln!(f, "   Drives: {} {}", disp_no_of_drives, "OK".red())?;
                }

                writeln!(f)?;
                continue;
            }

            if self.only_offline {
                continue;
            }

            writeln!(f, "{}  {}", colored_dot, srv.endpoint.blue())?;
            let uptime = humantime::format_duration(Duration::from_secs(srv.uptime as u64));
            writeln!(f, "   Uptime: {}", uptime)?;

            let version = if srv.version.contains("DEVELOPMENT") {
                "<development>"
            } else {
                &srv.version
            };
            writeln!(f, "   Version: {}", version)?;

            if let Some(network) = &srv.network {
                if backend_type == BackendType::Erasure {
                    let connection_alive = network.iter().filter(|&(_, v)| v == "online").count();
                    let display_nw_info = format!("{}/{}", connection_alive, network.len());
                    let clr = if connection_alive == network.len() {
                        "green"
                    } else {
                        "yellow"
                    };
                    writeln!(f, "   Network: {} {}", display_nw_info, "OK".color(clr))?;
                }
            }

            if backend_type == BackendType::Erasure {
                let (mut on_drives, mut off_drives) = (0, 0);
                if let Some(drives) = &srv.disks {
                    for disk in drives {
                        if disk.state == "ok" || disk.state == "unformatted" {
                            on_drives += 1;
                        } else {
                            off_drives += 1;
                        }
                    }
                }

                let disp_no_of_drives = format!("{}/{}", on_drives, on_drives + off_drives);
                let clr = if on_drives == on_drives + off_drives {
                    "green"
                } else {
                    "yellow"
                };
                writeln!(f, "   Drives: {} {}", disp_no_of_drives, "OK".color(clr))?;

                let pretty_pools: Vec<String> = endpoint_to_pools(&srv.endpoint, &cluster_summary)
                    .iter()
                    .map(|&pool| (pool + 1).to_string())
                    .collect();
                writeln!(f, "   Pool: {}", pretty_pools.join(", ").green())?;
            }

            writeln!(f)?;
        }

        let dsp_order = vec!["green"; cluster_summary.len() + 1];

        //let mut builder = String::new();
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        // 添加表头
        table.add_row(Row::new(vec![
            Cell::new("Pool").style_spec("Fg"),
            Cell::new("Drives Usage").style_spec("Fg"),
            Cell::new("Erasure stripe size").style_spec("Fg"),
            Cell::new("Erasure sets").style_spec("Fg"),
        ]));

        // 遍历池并添加数据行
        for (pool_idx, summary) in cluster_summary.iter() {
            let total_size = summary.drives_total_usable_space;
            let used_current = total_size - summary.drives_total_free_space;
            let capacity = if total_size == 0 {
                "0% (total: 0B)".to_string()
            } else {
                format!(
                    "{:.1}% (total: {})",
                    100.0 * used_current as f64 / total_size as f64,
                    human_bytes(total_size as f64).to_string()
                )
            };

            table.add_row(Row::new(vec![
                Cell::new(&(pool_idx + 1).to_string()).style_spec("Fg"),
                Cell::new(&capacity).style_spec("Fg"),
                Cell::new(&summary.drives_per_set.to_string()).style_spec("Fg"),
                Cell::new(&summary.sets_count.to_string()).style_spec("Fg"),
            ]));
        }

        // 将表格内容写入 builder
        writeln!(f, "{}", table).expect("Failed to write table");

        // 输出总结信息
        //let used_total = humanize::format_size(info.usage.size as u64);
        let used_total = info.usage.as_ref().unwrap().size;
        if info.buckets.as_ref().unwrap().count > 0 {
            writeln!(
                f,
                "{} Used, {}, {}",
                human_bytes(used_total as f64),
                pluralize(
                    "Bucket",
                    info.buckets.as_ref().unwrap().count as isize,
                    true
                ),
                pluralize(
                    "Object",
                    info.objects.as_ref().unwrap().count as isize,
                    true
                )
            )?;
            if info.versions.as_ref().unwrap().count > 0 {
                writeln!(
                    f,
                    ", {}",
                    pluralize(
                        "Version",
                        info.versions.as_ref().unwrap().count as isize,
                        false
                    )
                )?;
            }
            if info.delete_markers.as_ref().unwrap().count > 0 {
                writeln!(
                    f,
                    ", {}",
                    pluralize(
                        "Delete Marker",
                        info.delete_markers.as_ref().unwrap().count as isize,
                        false
                    )
                )?;
            }
            writeln!(f)?;
        }
        if backend_type == BackendType::Erasure {
            //println!("sfdsafdsafdsaf {}",info.backend.as_ref().unwrap().online_disks);
            if total_offline_nodes != 0 {
                writeln!(
                    f,
                    "{} offline, ",
                    pluralize("node", total_offline_nodes, false)
                )?;
            }
            writeln!(
                f,
                "{} online, {} offline, EC:{}",
                pluralize(
                    "drive",
                    info.backend.as_ref().unwrap().online_disks as isize,
                    true
                ),
                pluralize(
                    "drive",
                    info.backend.as_ref().unwrap().offline_disks as isize,
                    true
                ),
                info.backend.as_ref().unwrap().standard_sc_parity
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PoolSummary {
    index: i32,
    endpoints: HashSet<String>,
    drive_tolerance: u32,
    drives_total_free_space: u64,
    drives_total_usable_space: u64,
    sets_count: u32,
    drives_per_set: u32,
}

pub type ClusterInfo = HashMap<i32, PoolSummary>;

// 假设 InfoMessage 结构体以及 Disk 结构体已经定义
pub fn cluster_summary_info(info: &InfoMessage) -> ClusterInfo {
    let mut summary: ClusterInfo = HashMap::new();

    for srv in &info.servers {
        let Some(drives) = &srv.disks else {
            todo!();
        };
        for disk in drives {
            if disk.pool_index < 0 {
                continue;
            }

            let pool_entry = summary
                .entry(disk.pool_index)
                .or_insert_with(|| PoolSummary {
                    index: disk.pool_index,
                    endpoints: HashSet::new(),
                    drive_tolerance: info.standard_parity() as u32,
                    drives_total_free_space: 0,
                    drives_total_usable_space: 0,
                    sets_count: 0,
                    drives_per_set: 0,
                });

            if disk.disk_index
                < (info.backend.as_ref().unwrap().drives_per_set[disk.pool_index as usize]
                    - info.backend.as_ref().unwrap().standard_sc_parity)
            {
                pool_entry.drives_total_free_space += disk.available_space.as_ref().unwrap();
                pool_entry.drives_total_usable_space += disk.total_space.as_ref().unwrap();
            }

            pool_entry.endpoints.insert(srv.endpoint.clone());
        }
    }

    for (idx, &total_sets) in info.backend.as_ref().unwrap().total_sets.iter().enumerate() {
        if let Some(pool) = summary.get_mut(&(idx as i32)) {
            pool.sets_count = total_sets as u32;
            pool.drives_per_set = info.backend.as_ref().unwrap().drives_per_set[idx] as u32;
        }
    }

    summary
}
