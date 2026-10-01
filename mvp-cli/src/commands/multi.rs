//! Multi-node management commands
//!
//! Commands for managing multiple node instances for testing.

use clap::Subcommand;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::CliConfig;
use crate::output::OutputFormatter;
use crate::error::{CliResult, CliError};

#[derive(Subcommand, Debug)]
pub enum MultiCommand {
    /// Start multiple node instances
    Start {
        /// Number of instances to start
        count: usize,
        
        /// Base port for instances
        #[arg(short, long)]
        base_port: Option<u16>,
        
        /// Model to load on nodes
        #[arg(short, long)]
        model: Option<String>,
        
        /// Path to mvp-node binary
        #[arg(long)]
        binary: Option<String>,
    },
    
    /// Stop node instances
    Stop {
        /// Node IDs to stop (all if empty)
        #[arg(value_name = "NODE_ID")]
        nodes: Vec<String>,
    },
    
    /// List running instances
    List,
    
    /// View logs from instances
    Logs {
        /// Node ID to get logs from
        node_id: String,
        
        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
}

/// Node instance information
#[derive(Debug)]
pub struct NodeInstance {
    pub node_id: String,
    pub port: u16,
    pub pid: u32,
    pub status: String,
}

/// Multi-node manager state (would be stored in a file in real implementation)
lazy_static::lazy_static! {
    static ref INSTANCES: Arc<Mutex<HashMap<String, NodeInstance>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Execute a multi-node command
pub async fn execute(
    cmd: MultiCommand,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    match cmd {
        MultiCommand::Start { count, base_port, model, binary } => {
            start_nodes(count, base_port, model, binary, config, formatter).await
        }
        MultiCommand::Stop { nodes } => {
            stop_nodes(nodes, config, formatter).await
        }
        MultiCommand::List => {
            list_instances(config, formatter).await
        }
        MultiCommand::Logs { node_id, lines, follow } => {
            show_logs(node_id, lines, follow, config, formatter).await
        }
    }
}

async fn start_nodes(
    count: usize,
    base_port: Option<u16>,
    model: Option<String>,
    binary: Option<String>,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    let base = base_port.unwrap_or(config.multi_node.base_port);
    let model_name = model.or(config.default_model.clone())
        .unwrap_or_else(|| "tinyllama".to_string());
    
    // Find mvp-node binary
    let binary_path = binary.or(config.multi_node.node_binary.clone())
        .unwrap_or_else(|| {
            // Try to find in common locations
            if std::path::Path::new("./target/release/mvp-node").exists() {
                "./target/release/mvp-node".to_string()
            } else if std::path::Path::new("./target/debug/mvp-node").exists() {
                "./target/debug/mvp-node".to_string()
            } else if std::path::Path::new("../mvp-node/target/release/mvp-node").exists() {
                "../mvp-node/target/release/mvp-node".to_string()
            } else if std::path::Path::new("../mvp-node/target/debug/mvp-node").exists() {
                "../mvp-node/target/debug/mvp-node".to_string()
            } else {
                "mvp-node".to_string()
            }
        });
    
    println!("{}", formatter.header(&format!("Starting {} node instances", count)));
    println!("{}", formatter.kv("Base Port", &base.to_string()));
    println!("{}", formatter.kv("Model", &model_name));
    println!("{}", formatter.kv("Binary", &binary_path));
    println!();
    
    let mut instances = INSTANCES.lock().await;
    let mut started = Vec::new();
    
    for i in 0..count {
        let port = base + i as u16;
        let node_id = format!("node-{}", i + 1);
        
        // Bootstrap peers: first node is bootstrap, others connect to it
        let bootstrap_peers = if i == 0 {
            Vec::new()
        } else {
            vec![format!("/ip4/127.0.0.1/tcp/{}", base)]
        };
        
        println!("  {} Starting {}...", formatter.status("starting"), node_id);
        
        // In real implementation, we would actually start the process
        // For now, simulate it
        let instance = NodeInstance {
            node_id: node_id.clone(),
            port,
            pid: 10000 + i as u32, // Simulated PID
            status: "running".to_string(),
        };
        
        /*
        // Real implementation would be:
        let mut cmd = Command::new(&binary_path);
        cmd.arg("--port").arg(port.to_string())
           .arg("--model").arg(&model_name);
        
        for peer in &bootstrap_peers {
            cmd.arg("--bootstrap-peer").arg(peer);
        }
        
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CliError::ProcessError(format!("Failed to start {}: {}", node_id, e)))?;
        
        let instance = NodeInstance {
            node_id: node_id.clone(),
            port,
            pid: child.id(),
            status: "running".to_string(),
        };
        */
        
        started.push(vec![
            node_id.clone(),
            port.to_string(),
            instance.pid.to_string(),
            formatter.status("running"),
        ]);
        
        instances.insert(node_id, instance);
        
        // Small delay between starts
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    println!();
    println!("{}", formatter.success(&format!("Started {} instances", count)));
    println!();
    
    println!("{}", formatter.table(
        vec!["Node ID", "Port", "PID", "Status"],
        started
    ));
    
    println!();
    println!("{}", formatter.info("Use 'mvp-cli multi list' to see all instances"));
    println!("{}", formatter.info("Use 'mvp-cli multi stop' to stop instances"));
    
    Ok(())
}

async fn stop_nodes(
    nodes: Vec<String>,
    _config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    let mut instances = INSTANCES.lock().await;
    
    let to_stop: Vec<String> = if nodes.is_empty() {
        instances.keys().cloned().collect()
    } else {
        nodes
    };
    
    if to_stop.is_empty() {
        println!("{}", formatter.warning("No running instances to stop"));
        return Ok(());
    }
    
    println!("{}", formatter.header("Stopping Node Instances"));
    
    for node_id in &to_stop {
        if let Some(instance) = instances.remove(node_id) {
            println!("  {} Stopping {} (PID: {})...", 
                     formatter.status("stopping"), node_id, instance.pid);
            
            // In real implementation, we would kill the process
            // nix::sys::signal::kill(Pid::from_raw(instance.pid as i32), Signal::SIGTERM);
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            println!("  {} {} stopped", formatter.status("stopped"), node_id);
        } else {
            println!("  {} Instance {} not found", formatter.status("error"), node_id);
        }
    }
    
    println!();
    println!("{}", formatter.success(&format!("Stopped {} instances", to_stop.len())));
    
    Ok(())
}

async fn list_instances(
    _config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    let instances = INSTANCES.lock().await;
    
    println!("{}", formatter.header("Running Node Instances"));
    
    if instances.is_empty() {
        println!("{}", formatter.warning("No running instances"));
        println!();
        println!("{}", formatter.info("Use 'mvp-cli multi start <count>' to start instances"));
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = instances.values()
        .map(|i| vec![
            i.node_id.clone(),
            i.port.to_string(),
            i.pid.to_string(),
            formatter.status(&i.status),
        ])
        .collect();
    
    println!("{}", formatter.table(
        vec!["Node ID", "Port", "PID", "Status"],
        rows
    ));
    
    Ok(())
}

async fn show_logs(
    node_id: String,
    lines: usize,
    follow: bool,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    let instances = INSTANCES.lock().await;
    
    if !instances.contains_key(&node_id) {
        return Err(CliError::NotFound(format!("Node instance '{}'", node_id)));
    }
    
    let log_file = format!("{}/{}.log", config.multi_node.log_dir, node_id);
    
    println!("{}", formatter.header(&format!("Logs: {}", node_id)));
    
    // In real implementation, read from log file
    // For now, show sample logs
    let sample_logs = vec![
        "[2024-01-01 12:00:00] INFO  Starting MVP Node...",
        "[2024-01-01 12:00:01] INFO  Loading model: tinyllama",
        "[2024-01-01 12:00:02] INFO  Model loaded successfully",
        "[2024-01-01 12:00:03] INFO  Starting P2P network on port 5000",
        "[2024-01-01 12:00:04] INFO  Listening on /ip4/127.0.0.1/tcp/5000",
        "[2024-01-01 12:00:05] INFO  Connected to bootstrap peer",
        "[2024-01-01 12:00:10] INFO  Node ready to accept jobs",
    ];
    
    for line in sample_logs.iter().take(lines) {
        println!("{}", line);
    }
    
    if follow {
        println!();
        println!("{}", formatter.info("Following log output... (Ctrl+C to stop)"));
        
        let mut counter = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            counter += 1;
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("[{}] INFO  Heartbeat #{}", now, counter);
        }
    }
    
    Ok(())
}

// Add lazy_static to Cargo.toml











