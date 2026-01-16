//! Firecracker Pool Load Testing Tool
//!
//! A comprehensive tool for load testing veritech's Firecracker pool.
//! Captures system state, generates load via NATS, and monitors resources.
//!
//! Usage:
//!   # Analyze system configuration only
//!   cargo run -p fc-load-test -- analyze
//!
//!   # Run load test with 1000 requests
//!   cargo run -p fc-load-test -- load \
//!     --nats-url localhost:4222 \
//!     --burst-size 1000 \
//!     --fast-pct 70 --medium-pct 20 --slow-pct 10
//!
//!   # Monitor system resources in real-time
//!   cargo run -p fc-load-test -- monitor --interval 1000

use std::{
    collections::HashMap,
    fs::{
        self,
        File,
    },
    io::Write,
    path::PathBuf,
    process::Command as StdCommand,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            AtomicU64,
            Ordering,
        },
    },
    time::{
        Duration,
        Instant,
    },
};

use async_nats::jetstream;
use base64::Engine;
use chrono::{
    DateTime,
    Utc,
};
use clap::{
    Parser,
    Subcommand,
};
use color_eyre::eyre::{
    Context,
    Result,
};
use futures::StreamExt;
use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};
use sysinfo::System;
use tokio::sync::mpsc;
use ulid::Ulid;

// ============================================================================
// CLI Definition
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "fc-load-test")]
#[command(about = "Firecracker pool load testing and system analysis tool")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze current system configuration for Firecracker
    Analyze {
        /// Output file for JSON results (default: stdout)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Human-readable output instead of JSON
        #[arg(long)]
        human: bool,
    },

    /// Run load test against veritech
    Load {
        /// NATS server URL
        #[arg(long, default_value = "localhost:4222")]
        nats_url: String,

        /// NATS subject prefix (optional)
        #[arg(long)]
        subject_prefix: Option<String>,

        /// Workspace ID for requests (generates random if not provided)
        #[arg(long)]
        workspace_id: Option<String>,

        /// Change set ID for requests (generates random if not provided)
        #[arg(long)]
        change_set_id: Option<String>,

        /// Number of requests to send
        #[arg(long, default_value = "100")]
        burst_size: usize,

        /// Maximum requests per second (0 = unlimited)
        #[arg(long, default_value = "0")]
        rate_limit: u32,

        /// Timeout in seconds for waiting for responses
        #[arg(long, default_value = "120")]
        timeout: u64,

        /// Percentage of fast functions (~100ms)
        #[arg(long, default_value = "70")]
        fast_pct: u8,

        /// Percentage of medium functions (~5s)
        #[arg(long, default_value = "20")]
        medium_pct: u8,

        /// Percentage of slow functions (~30s)
        #[arg(long, default_value = "10")]
        slow_pct: u8,

        /// Capture system state before and after test
        #[arg(long)]
        capture_system: bool,

        /// Sample system resources during test (interval in ms, 0 = disabled)
        #[arg(long, default_value = "1000")]
        sample_interval: u64,

        /// Output file for JSON results
        #[arg(long)]
        output: Option<PathBuf>,

        /// Show verbose progress
        #[arg(long, short)]
        verbose: bool,
    },

    /// Monitor system resources in real-time
    Monitor {
        /// Sampling interval in milliseconds
        #[arg(long, default_value = "1000")]
        interval: u64,

        /// Output directory for CSV files
        #[arg(long, default_value = "./fc-monitor-data")]
        output_dir: PathBuf,

        /// Stop after N seconds (0 = run until Ctrl+C)
        #[arg(long, default_value = "0")]
        duration: u64,
    },
}

// ============================================================================
// System Analysis Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct SystemSnapshot {
    timestamp: DateTime<Utc>,
    hostname: String,
    resources: ResourceInfo,
    kernel_params: HashMap<String, String>,
    cgroups: CgroupInfo,
    jails: JailInfo,
    bottleneck_analysis: Vec<BottleneckAnalysis>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourceInfo {
    loop_devices: LoopDeviceInfo,
    dm_devices: DeviceMapperInfo,
    network_namespaces: u32,
    firecracker_processes: u32,
    jailer_processes: u32,
    memory: MemoryInfo,
    cpu: CpuInfo,
    file_descriptors: FdInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoopDeviceInfo {
    max: u32,
    in_use: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceMapperInfo {
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryInfo {
    total_mb: u64,
    used_mb: u64,
    available_mb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CpuInfo {
    cores: u32,
    usage_percent: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FdInfo {
    max: u64,
    used: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CgroupInfo {
    veritech_cpus: Option<String>,
    firecracker_cpus: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JailInfo {
    count: u32,
    disk_usage_mb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BottleneckAnalysis {
    resource: String,
    status: String, // "ok", "warning", "critical"
    usage_percent: f32,
    message: String,
}

// ============================================================================
// Load Test Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolverFunctionRequest {
    execution_id: String,
    handler: String,
    component: ResolverFunctionComponent,
    response_type: String,
    code_base64: String,
    before: Vec<()>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolverFunctionComponent {
    data: ComponentView,
    parents: Vec<ComponentView>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComponentView {
    kind: String,
    properties: serde_json::Value,
}

#[derive(Debug, Clone, Copy)]
enum FunctionDuration {
    Fast,   // ~100ms CPU work
    Medium, // ~5s CPU work
    Slow,   // ~30s CPU work
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadTestConfig {
    burst_size: usize,
    rate_limit: u32,
    timeout_secs: u64,
    function_mix: FunctionMix,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionMix {
    fast_pct: u8,
    medium_pct: u8,
    slow_pct: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadTestResults {
    config: LoadTestConfig,
    summary: ResultsSummary,
    latency: LatencyStats,
    samples: Vec<ResourceSample>,
    system_before: Option<SystemSnapshot>,
    system_after: Option<SystemSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResultsSummary {
    total_requests: usize,
    successful: usize,
    failed: usize,
    timed_out: usize,
    duration_secs: f64,
    throughput_rps: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatencyStats {
    min_ms: u64,
    max_ms: u64,
    avg_ms: f64,
    p50_ms: u64,
    p95_ms: u64,
    p99_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceSample {
    timestamp_ms: u64,
    loop_devices: u32,
    dm_devices: u32,
    fc_processes: u32,
    memory_used_mb: u64,
    cpu_percent: f32,
}

#[derive(Debug)]
struct RequestResult {
    execution_id: String,
    success: bool,
    latency_ms: u64,
    error: Option<String>,
}

// ============================================================================
// System Collection Functions
// ============================================================================

fn collect_system_snapshot() -> Result<SystemSnapshot> {
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());

    let resources = collect_resources()?;
    let kernel_params = collect_kernel_params();
    let cgroups = collect_cgroups();
    let jails = collect_jails();
    let bottleneck_analysis = analyze_bottlenecks(&resources);

    Ok(SystemSnapshot {
        timestamp: Utc::now(),
        hostname,
        resources,
        kernel_params,
        cgroups,
        jails,
        bottleneck_analysis,
    })
}

fn collect_resources() -> Result<ResourceInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Loop devices
    let loop_max = read_file_u32("/sys/module/loop/parameters/max_loop").unwrap_or(256);
    let loop_in_use = count_loop_devices();

    // Device mapper
    let dm_count = count_dm_devices();

    // Network namespaces
    let netns_count = count_network_namespaces();

    // Processes
    let fc_processes = count_processes("firecracker");
    let jailer_processes = count_processes("jailer");

    // Memory
    let total_mem = sys.total_memory() / 1024 / 1024;
    let used_mem = sys.used_memory() / 1024 / 1024;
    let available_mem = sys.available_memory() / 1024 / 1024;

    // CPU
    let cpu_cores = sys.cpus().len() as u32;
    let cpu_usage = sys.global_cpu_usage();

    // File descriptors
    let (fd_max, fd_used) = get_fd_info();

    Ok(ResourceInfo {
        loop_devices: LoopDeviceInfo {
            max: loop_max,
            in_use: loop_in_use,
        },
        dm_devices: DeviceMapperInfo { count: dm_count },
        network_namespaces: netns_count,
        firecracker_processes: fc_processes,
        jailer_processes: jailer_processes,
        memory: MemoryInfo {
            total_mb: total_mem,
            used_mb: used_mem,
            available_mb: available_mem,
        },
        cpu: CpuInfo {
            cores: cpu_cores,
            usage_percent: cpu_usage,
        },
        file_descriptors: FdInfo {
            max: fd_max,
            used: fd_used,
        },
    })
}

fn read_file_u32(path: &str) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn count_loop_devices() -> u32 {
    let output = StdCommand::new("losetup").arg("-l").output().ok();
    match output {
        Some(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Subtract 1 for header line if present
            let lines = stdout.lines().count();
            if lines > 0 { (lines - 1) as u32 } else { 0 }
        }
        None => 0,
    }
}

fn count_dm_devices() -> u32 {
    let output = StdCommand::new("dmsetup").arg("ls").output().ok();
    match output {
        Some(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("No devices found") {
                0
            } else {
                stdout.lines().count() as u32
            }
        }
        None => 0,
    }
}

fn count_network_namespaces() -> u32 {
    let output = StdCommand::new("ip").args(["netns", "list"]).output().ok();
    match output {
        Some(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().count() as u32
        }
        None => 0,
    }
}

fn count_processes(name: &str) -> u32 {
    let output = StdCommand::new("pgrep").arg("-c").arg(name).output().ok();
    match output {
        Some(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.trim().parse().unwrap_or(0)
        }
        None => 0,
    }
}

fn get_fd_info() -> (u64, u64) {
    let max = fs::read_to_string("/proc/sys/fs/file-max")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let used = fs::read_to_string("/proc/sys/fs/file-nr")
        .ok()
        .and_then(|s| s.split_whitespace().next().and_then(|n| n.parse().ok()))
        .unwrap_or(0);

    (max, used)
}

fn collect_kernel_params() -> HashMap<String, String> {
    let params = [
        "net.core.somaxconn",
        "net.ipv4.tcp_max_syn_backlog",
        "net.ipv4.neigh.default.gc_thresh3",
        "net.ipv4.ip_local_port_range",
        "net.ipv4.tcp_fin_timeout",
        "net.ipv4.tcp_tw_reuse",
    ];

    let mut result = HashMap::new();
    for param in params {
        if let Ok(output) = StdCommand::new("sysctl").arg("-n").arg(param).output() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                result.insert(param.to_string(), value);
            }
        }
    }
    result
}

fn collect_cgroups() -> CgroupInfo {
    let veritech_cpus = fs::read_to_string("/sys/fs/cgroup/veritech/cpuset.cpus")
        .ok()
        .map(|s| s.trim().to_string());

    let firecracker_cpus = fs::read_to_string("/sys/fs/cgroup/veritech/firecracker/cpuset.cpus")
        .ok()
        .map(|s| s.trim().to_string());

    CgroupInfo {
        veritech_cpus,
        firecracker_cpus,
    }
}

fn collect_jails() -> JailInfo {
    let jail_path = "/srv/jailer/firecracker";
    let count = fs::read_dir(jail_path)
        .map(|entries| entries.count() as u32)
        .unwrap_or(0);

    // Get disk usage with du
    let disk_usage = StdCommand::new("du")
        .args(["-sm", jail_path])
        .output()
        .ok()
        .and_then(|out| {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);

    JailInfo {
        count,
        disk_usage_mb: disk_usage,
    }
}

fn analyze_bottlenecks(resources: &ResourceInfo) -> Vec<BottleneckAnalysis> {
    let mut analysis = Vec::new();

    // Loop devices
    let loop_usage = if resources.loop_devices.max > 0 {
        (resources.loop_devices.in_use as f32 / resources.loop_devices.max as f32) * 100.0
    } else {
        0.0
    };
    analysis.push(BottleneckAnalysis {
        resource: "loop_devices".to_string(),
        status: if loop_usage > 90.0 {
            "critical"
        } else if loop_usage > 75.0 {
            "warning"
        } else {
            "ok"
        }
        .to_string(),
        usage_percent: loop_usage,
        message: format!(
            "{}/{} loop devices in use ({:.1}%)",
            resources.loop_devices.in_use, resources.loop_devices.max, loop_usage
        ),
    });

    // Memory
    let mem_usage = if resources.memory.total_mb > 0 {
        (resources.memory.used_mb as f32 / resources.memory.total_mb as f32) * 100.0
    } else {
        0.0
    };
    analysis.push(BottleneckAnalysis {
        resource: "memory".to_string(),
        status: if mem_usage > 95.0 {
            "critical"
        } else if mem_usage > 85.0 {
            "warning"
        } else {
            "ok"
        }
        .to_string(),
        usage_percent: mem_usage,
        message: format!(
            "{} MB used / {} MB total ({:.1}%), {} MB available",
            resources.memory.used_mb,
            resources.memory.total_mb,
            mem_usage,
            resources.memory.available_mb
        ),
    });

    // File descriptors
    let fd_usage = if resources.file_descriptors.max > 0 {
        (resources.file_descriptors.used as f32 / resources.file_descriptors.max as f32) * 100.0
    } else {
        0.0
    };
    analysis.push(BottleneckAnalysis {
        resource: "file_descriptors".to_string(),
        status: if fd_usage > 90.0 {
            "critical"
        } else if fd_usage > 75.0 {
            "warning"
        } else {
            "ok"
        }
        .to_string(),
        usage_percent: fd_usage,
        message: format!(
            "{}/{} file descriptors in use ({:.1}%)",
            resources.file_descriptors.used, resources.file_descriptors.max, fd_usage
        ),
    });

    analysis
}

fn print_human_readable(snapshot: &SystemSnapshot) {
    println!("=== Firecracker System Analysis ===");
    println!("Timestamp: {}", snapshot.timestamp);
    println!("Hostname:  {}", snapshot.hostname);
    println!();

    println!("=== Resources ===");
    println!(
        "Loop devices:     {}/{} ({:.1}% used)",
        snapshot.resources.loop_devices.in_use,
        snapshot.resources.loop_devices.max,
        (snapshot.resources.loop_devices.in_use as f32
            / snapshot.resources.loop_devices.max as f32)
            * 100.0
    );
    println!("DM devices:       {}", snapshot.resources.dm_devices.count);
    println!(
        "Network NS:       {}",
        snapshot.resources.network_namespaces
    );
    println!(
        "FC processes:     {}",
        snapshot.resources.firecracker_processes
    );
    println!("Jailer processes: {}", snapshot.resources.jailer_processes);
    println!();

    println!("=== Memory ===");
    println!("Total:     {} MB", snapshot.resources.memory.total_mb);
    println!("Used:      {} MB", snapshot.resources.memory.used_mb);
    println!("Available: {} MB", snapshot.resources.memory.available_mb);
    println!();

    println!("=== CPU ===");
    println!("Cores:  {}", snapshot.resources.cpu.cores);
    println!("Usage:  {:.1}%", snapshot.resources.cpu.usage_percent);
    println!();

    println!("=== Jails ===");
    println!("Count:      {}", snapshot.jails.count);
    println!("Disk usage: {} MB", snapshot.jails.disk_usage_mb);
    println!();

    println!("=== Cgroups ===");
    println!(
        "Veritech CPUs:     {}",
        snapshot.cgroups.veritech_cpus.as_deref().unwrap_or("N/A")
    );
    println!(
        "Firecracker CPUs:  {}",
        snapshot
            .cgroups
            .firecracker_cpus
            .as_deref()
            .unwrap_or("N/A")
    );
    println!();

    println!("=== Bottleneck Analysis ===");
    for analysis in &snapshot.bottleneck_analysis {
        let status_icon = match analysis.status.as_str() {
            "ok" => "[OK]",
            "warning" => "[WARN]",
            "critical" => "[CRIT]",
            _ => "[??]",
        };
        println!(
            "{} {}: {}",
            status_icon, analysis.resource, analysis.message
        );
    }
}

// ============================================================================
// Load Test Functions
// ============================================================================

fn veritech_subject(prefix: Option<&str>, workspace_id: &str, change_set_id: &str) -> String {
    match prefix {
        Some(p) => {
            format!("{p}.veritech.requests.{workspace_id}.{change_set_id}.resolverfunction")
        }
        None => format!("veritech.requests.{workspace_id}.{change_set_id}.resolverfunction"),
    }
}

fn veritech_stream_name(prefix: Option<&str>) -> String {
    match prefix {
        Some(p) => format!("{p}_VERITECH_REQUESTS"),
        None => "VERITECH_REQUESTS".to_string(),
    }
}

fn generate_function_code(duration: FunctionDuration) -> String {
    let iterations = match duration {
        FunctionDuration::Fast => 1_000_000,    // ~100ms
        FunctionDuration::Medium => 50_000_000, // ~5s
        FunctionDuration::Slow => 300_000_000,  // ~30s
    };

    let code = format!(
        r#"function resolver(input) {{
    let result = 0;
    for (let i = 0; i < {iterations}; i++) {{
        result += Math.sqrt(i) * Math.sin(i);
    }}
    return {{ value: result, iterations: {iterations} }};
}}"#
    );

    base64::engine::general_purpose::STANDARD.encode(code)
}

fn pick_function_duration(fast_pct: u8, medium_pct: u8) -> FunctionDuration {
    let roll: u8 = rand::thread_rng().gen_range(0..100);
    if roll < fast_pct {
        FunctionDuration::Fast
    } else if roll < fast_pct + medium_pct {
        FunctionDuration::Medium
    } else {
        FunctionDuration::Slow
    }
}

fn create_request(duration: FunctionDuration) -> ResolverFunctionRequest {
    ResolverFunctionRequest {
        execution_id: Ulid::new().to_string(),
        handler: "resolver".to_string(),
        component: ResolverFunctionComponent {
            data: ComponentView {
                kind: "standard".to_string(),
                properties: serde_json::json!({}),
            },
            parents: vec![],
        },
        response_type: "Json".to_string(),
        code_base64: generate_function_code(duration),
        before: vec![],
    }
}

async fn run_load_test(
    nats_url: &str,
    subject_prefix: Option<&str>,
    workspace_id: &str,
    change_set_id: &str,
    burst_size: usize,
    rate_limit: u32,
    timeout_secs: u64,
    fast_pct: u8,
    medium_pct: u8,
    sample_interval: u64,
    capture_system: bool,
    verbose: bool,
) -> Result<LoadTestResults> {
    // Capture system state before
    let system_before = if capture_system {
        println!("Capturing system state (before)...");
        Some(collect_system_snapshot()?)
    } else {
        None
    };

    // Connect to NATS
    println!("Connecting to NATS at {}...", nats_url);
    let client = async_nats::connect(nats_url)
        .await
        .context("Failed to connect to NATS")?;
    let jetstream = jetstream::new(client.clone());

    // Verify stream exists
    let stream_name = veritech_stream_name(subject_prefix);
    println!("Checking for veritech stream '{}'...", stream_name);
    match jetstream.get_stream(&stream_name).await {
        Ok(mut stream) => {
            let info = stream.info().await?;
            println!(
                "  Found! {} messages, {} consumers",
                info.state.messages, info.state.consumer_count
            );
        }
        Err(e) => {
            return Err(color_eyre::eyre::eyre!(
                "Could not find stream '{}': {}. Is veritech running?",
                stream_name,
                e
            ));
        }
    }

    let subject = veritech_subject(subject_prefix, workspace_id, change_set_id);
    println!("Publishing to subject: {}", subject);
    println!();

    // Start resource sampling if enabled
    let samples = Arc::new(tokio::sync::Mutex::new(Vec::<ResourceSample>::new()));
    let sampling_active = Arc::new(AtomicBool::new(true));

    let sampler_handle = if sample_interval > 0 {
        let samples_clone = Arc::clone(&samples);
        let active_clone = Arc::clone(&sampling_active);
        let interval = sample_interval;

        Some(tokio::spawn(async move {
            let start = Instant::now();
            while active_clone.load(Ordering::Relaxed) {
                let sample = ResourceSample {
                    timestamp_ms: start.elapsed().as_millis() as u64,
                    loop_devices: count_loop_devices(),
                    dm_devices: count_dm_devices(),
                    fc_processes: count_processes("firecracker"),
                    memory_used_mb: {
                        let mut sys = System::new();
                        sys.refresh_memory();
                        sys.used_memory() / 1024 / 1024
                    },
                    cpu_percent: {
                        let mut sys = System::new();
                        sys.refresh_cpu_all();
                        sys.global_cpu_usage()
                    },
                };
                samples_clone.lock().await.push(sample);
                tokio::time::sleep(Duration::from_millis(interval)).await;
            }
        }))
    } else {
        None
    };

    // Track results
    let (result_tx, mut result_rx) = mpsc::channel::<RequestResult>(burst_size);
    let pending = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();

    println!(
        "=== Sending {} requests ({}% fast, {}% medium, {}% slow) ===",
        burst_size,
        fast_pct,
        medium_pct,
        100 - fast_pct - medium_pct
    );

    // Send requests
    for i in 0..burst_size {
        let duration = pick_function_duration(fast_pct, medium_pct);
        let request = create_request(duration);
        let execution_id = request.execution_id.clone();
        let payload = serde_json::to_vec(&request)?;

        let reply_inbox = client.new_inbox();
        let mut reply_sub = client.subscribe(format!("{}.>", &reply_inbox)).await?;

        let mut headers = async_nats::HeaderMap::new();
        headers.insert("X-Reply-Inbox", reply_inbox.as_str());

        let send_time = Instant::now();
        pending.fetch_add(1, Ordering::Relaxed);

        match jetstream
            .publish_with_headers(subject.clone(), headers, payload.into())
            .await
        {
            Ok(ack) => {
                if let Err(e) = ack.await {
                    result_tx
                        .send(RequestResult {
                            execution_id,
                            success: false,
                            latency_ms: 0,
                            error: Some(format!("Ack failed: {}", e)),
                        })
                        .await
                        .ok();
                    pending.fetch_sub(1, Ordering::Relaxed);
                    continue;
                }
            }
            Err(e) => {
                result_tx
                    .send(RequestResult {
                        execution_id,
                        success: false,
                        latency_ms: 0,
                        error: Some(format!("Publish failed: {}", e)),
                    })
                    .await
                    .ok();
                pending.fetch_sub(1, Ordering::Relaxed);
                continue;
            }
        }

        // Spawn task to wait for response
        let tx = result_tx.clone();
        let pending_clone = Arc::clone(&pending);
        let timeout = Duration::from_secs(timeout_secs);
        let exec_id = execution_id.clone();

        tokio::spawn(async move {
            let result = tokio::time::timeout(timeout, async {
                while let Some(msg) = reply_sub.next().await {
                    // Check for final message
                    if let Some(headers) = &msg.headers {
                        if headers.get("X-Final-Message").is_some() {
                            return Some(send_time.elapsed());
                        }
                    }
                    // Also check subject for .result suffix
                    if msg.subject.ends_with(".result") {
                        return Some(send_time.elapsed());
                    }
                }
                None
            })
            .await;

            let request_result = match result {
                Ok(Some(elapsed)) => RequestResult {
                    execution_id: exec_id,
                    success: true,
                    latency_ms: elapsed.as_millis() as u64,
                    error: None,
                },
                Ok(None) => RequestResult {
                    execution_id: exec_id,
                    success: false,
                    latency_ms: 0,
                    error: Some("Stream closed without result".to_string()),
                },
                Err(_) => RequestResult {
                    execution_id: exec_id,
                    success: false,
                    latency_ms: 0,
                    error: Some("Timeout".to_string()),
                },
            };

            pending_clone.fetch_sub(1, Ordering::Relaxed);
            tx.send(request_result).await.ok();
        });

        // Rate limiting
        if rate_limit > 0 && i < burst_size - 1 {
            let delay = Duration::from_secs_f64(1.0 / rate_limit as f64);
            tokio::time::sleep(delay).await;
        }

        // Progress reporting
        if verbose && (i + 1) % 100 == 0 {
            println!("  Sent {}/{} requests...", i + 1, burst_size);
        }
    }

    println!("All requests sent. Waiting for responses...");

    // Drop the sender so the receiver knows when all senders are done
    drop(result_tx);

    // Collect results
    let mut results = Vec::with_capacity(burst_size);
    while let Some(result) = result_rx.recv().await {
        results.push(result);
        if verbose && results.len() % 100 == 0 {
            println!("  Received {}/{} responses...", results.len(), burst_size);
        }
    }

    let total_duration = start_time.elapsed();

    // Stop sampling
    sampling_active.store(false, Ordering::Relaxed);
    if let Some(handle) = sampler_handle {
        handle.abort();
    }

    // Compute statistics
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results
        .iter()
        .filter(|r| !r.success && r.error.as_deref() != Some("Timeout"))
        .count();
    let timed_out = results
        .iter()
        .filter(|r| r.error.as_deref() == Some("Timeout"))
        .count();

    let mut latencies: Vec<u64> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.latency_ms)
        .collect();
    latencies.sort();

    let latency = if !latencies.is_empty() {
        let sum: u64 = latencies.iter().sum();
        let len = latencies.len();
        LatencyStats {
            min_ms: latencies[0],
            max_ms: latencies[len - 1],
            avg_ms: sum as f64 / len as f64,
            p50_ms: latencies[len / 2],
            p95_ms: latencies[(len as f64 * 0.95) as usize],
            p99_ms: latencies[(len as f64 * 0.99) as usize],
        }
    } else {
        LatencyStats {
            min_ms: 0,
            max_ms: 0,
            avg_ms: 0.0,
            p50_ms: 0,
            p95_ms: 0,
            p99_ms: 0,
        }
    };

    let duration_secs = total_duration.as_secs_f64();
    let throughput_rps = if duration_secs > 0.0 {
        successful as f64 / duration_secs
    } else {
        0.0
    };

    // Capture system state after
    let system_after = if capture_system {
        println!("Capturing system state (after)...");
        Some(collect_system_snapshot()?)
    } else {
        None
    };

    let collected_samples = samples.lock().await.clone();

    Ok(LoadTestResults {
        config: LoadTestConfig {
            burst_size,
            rate_limit,
            timeout_secs,
            function_mix: FunctionMix {
                fast_pct,
                medium_pct,
                slow_pct: 100 - fast_pct - medium_pct,
            },
        },
        summary: ResultsSummary {
            total_requests: burst_size,
            successful,
            failed,
            timed_out,
            duration_secs,
            throughput_rps,
        },
        latency,
        samples: collected_samples,
        system_before,
        system_after,
    })
}

fn print_load_test_results(results: &LoadTestResults) {
    println!();
    println!("=== Load Test Results ===");
    println!();
    println!("Configuration:");
    println!("  Burst size:    {}", results.config.burst_size);
    println!(
        "  Function mix:  {}% fast, {}% medium, {}% slow",
        results.config.function_mix.fast_pct,
        results.config.function_mix.medium_pct,
        results.config.function_mix.slow_pct
    );
    println!();

    println!("Summary:");
    println!("  Total requests: {}", results.summary.total_requests);
    println!(
        "  Successful:     {} ({:.1}%)",
        results.summary.successful,
        (results.summary.successful as f64 / results.summary.total_requests as f64) * 100.0
    );
    println!("  Failed:         {}", results.summary.failed);
    println!("  Timed out:      {}", results.summary.timed_out);
    println!("  Duration:       {:.2}s", results.summary.duration_secs);
    println!(
        "  Throughput:     {:.2} req/s",
        results.summary.throughput_rps
    );
    println!();

    println!("Latency (successful requests):");
    println!("  Min:  {} ms", results.latency.min_ms);
    println!("  Avg:  {:.1} ms", results.latency.avg_ms);
    println!("  P50:  {} ms", results.latency.p50_ms);
    println!("  P95:  {} ms", results.latency.p95_ms);
    println!("  P99:  {} ms", results.latency.p99_ms);
    println!("  Max:  {} ms", results.latency.max_ms);

    if !results.samples.is_empty() {
        println!();
        println!("Resource samples collected: {}", results.samples.len());
    }
}

// ============================================================================
// Monitor Functions
// ============================================================================

async fn run_monitor(output_dir: PathBuf, interval: u64, duration: u64) -> Result<()> {
    fs::create_dir_all(&output_dir)?;

    let csv_path = output_dir.join("resources.csv");
    let mut file = File::create(&csv_path)?;
    writeln!(
        file,
        "timestamp_ms,loop_devices,dm_devices,netns_count,fc_processes,memory_used_mb,cpu_percent"
    )?;

    println!("=== Firecracker Resource Monitor ===");
    println!("Output: {}", csv_path.display());
    println!("Interval: {}ms", interval);
    if duration > 0 {
        println!("Duration: {}s", duration);
    } else {
        println!("Duration: unlimited (Ctrl+C to stop)");
    }
    println!();

    let start = Instant::now();
    let mut last_report = Instant::now();

    loop {
        let elapsed = start.elapsed();

        if duration > 0 && elapsed.as_secs() >= duration {
            println!("\nDuration reached. Stopping.");
            break;
        }

        // Collect sample
        let loop_devices = count_loop_devices();
        let dm_devices = count_dm_devices();
        let netns_count = count_network_namespaces();
        let fc_processes = count_processes("firecracker");

        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu_all();
        let memory_used_mb = sys.used_memory() / 1024 / 1024;
        let cpu_percent = sys.global_cpu_usage();

        // Write to CSV
        writeln!(
            file,
            "{},{},{},{},{},{},{:.1}",
            elapsed.as_millis(),
            loop_devices,
            dm_devices,
            netns_count,
            fc_processes,
            memory_used_mb,
            cpu_percent
        )?;
        file.flush()?;

        // Periodic console output
        if last_report.elapsed().as_secs() >= 5 {
            println!(
                "[{:>6.1}s] loops={} dm={} netns={} fc={} mem={}MB cpu={:.1}%",
                elapsed.as_secs_f64(),
                loop_devices,
                dm_devices,
                netns_count,
                fc_processes,
                memory_used_mb,
                cpu_percent
            );
            last_report = Instant::now();
        }

        tokio::time::sleep(Duration::from_millis(interval)).await;
    }

    println!("\nData written to: {}", csv_path.display());
    Ok(())
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.command {
        Commands::Analyze { output, human } => {
            let snapshot = collect_system_snapshot()?;

            if human {
                print_human_readable(&snapshot);
            } else {
                let json = serde_json::to_string_pretty(&snapshot)?;
                if let Some(path) = output {
                    fs::write(&path, &json)?;
                    println!("Written to: {}", path.display());
                } else {
                    println!("{}", json);
                }
            }
        }

        Commands::Load {
            nats_url,
            subject_prefix,
            workspace_id,
            change_set_id,
            burst_size,
            rate_limit,
            timeout,
            fast_pct,
            medium_pct,
            slow_pct,
            capture_system,
            sample_interval,
            output,
            verbose,
        } => {
            // Validate percentages
            if fast_pct + medium_pct + slow_pct != 100 {
                return Err(color_eyre::eyre::eyre!(
                    "Function percentages must sum to 100 (got {})",
                    fast_pct + medium_pct + slow_pct
                ));
            }

            // Generate IDs if not provided
            let workspace_id = workspace_id.unwrap_or_else(|| Ulid::new().to_string());
            let change_set_id = change_set_id.unwrap_or_else(|| Ulid::new().to_string());

            let results = run_load_test(
                &nats_url,
                subject_prefix.as_deref(),
                &workspace_id,
                &change_set_id,
                burst_size,
                rate_limit,
                timeout,
                fast_pct,
                medium_pct,
                sample_interval,
                capture_system,
                verbose,
            )
            .await?;

            print_load_test_results(&results);

            if let Some(path) = output {
                let json = serde_json::to_string_pretty(&results)?;
                fs::write(&path, &json)?;
                println!("\nResults written to: {}", path.display());
            }
        }

        Commands::Monitor {
            interval,
            output_dir,
            duration,
        } => {
            run_monitor(output_dir, interval, duration).await?;
        }
    }

    Ok(())
}
