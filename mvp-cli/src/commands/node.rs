//! Node command handlers
//!
//! Commands for monitoring and querying nodes via real P2P.
//!
//! Implements Requirements 4.1-4.5

use clap::Subcommand;
use crate::config::CliConfig;
use crate::output::OutputFormatter;
use crate::error::{CliResult, CliError};
use crate::client::NodeClient;
use std::time::Duration;
use tracing::info;

#[derive(Subcommand, Debug)]
pub enum NodeCommand {
    /// List all discovered nodes
    List {
        /// Continuously watch for changes
        #[arg(short, long)]
        watch: bool,
        
        /// Refresh interval in seconds (with --watch)
        #[arg(long, default_value = "5")]
        interval: u64,
        
        /// Discovery timeout in seconds
        #[arg(long, default_value = "3")]
        discovery_timeout: u64,
    },
    
    /// Get detailed information about a node
    Info {
        /// Node ID or Peer ID
        node_id: String,
    },
    
    /// Check health status of a node
    Health {
        /// Node ID or Peer ID
        node_id: String,
    },
    
    /// View node metrics
    Metrics {
        /// Node ID or Peer ID
        node_id: String,
        
        /// Specific metric type to show
        #[arg(short, long)]
        metric: Option<String>,
        
        /// Continuously watch metrics
        #[arg(short, long)]
        watch: bool,
    },
    
    /// Ping a node to check latency
    Ping {
        /// Node ID or Peer ID
        node_id: String,
        
        /// Number of pings to send
        #[arg(short, long, default_value = "3")]
        count: u32,
    },
}

/// Execute a node command
pub async fn execute(
    cmd: NodeCommand,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    // Create P2P client
    let mut client = NodeClient::new(config.timeout_secs).await?;
    
    // Start listening
    let addr = client.start().await?;
    info!("CLI started on {}", addr);
    
    match cmd {
        NodeCommand::List { watch, interval, discovery_timeout } => {
            list_nodes(&mut client, watch, interval, discovery_timeout, formatter).await
        }
        NodeCommand::Info { node_id } => {
            get_node_info(&mut client, node_id, formatter).await
        }
        NodeCommand::Health { node_id } => {
            check_health(&mut client, node_id, formatter).await
        }
        NodeCommand::Metrics { node_id, metric, watch } => {
            show_metrics(&mut client, node_id, metric, watch, formatter).await
        }
        NodeCommand::Ping { node_id, count } => {
            ping_node(&mut client, node_id, count, formatter).await
        }
    }
}

/// List discovered nodes
async fn list_nodes(
    client: &mut NodeClient,
    watch: bool,
    interval: u64,
    discovery_timeout: u64,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    loop {
        if watch {
            // Clear screen for watch mode
            print!("\x1B[2J\x1B[1;1H");
        }
        
        println!("{}", formatter.header("Discovering Nodes..."));
        
        // Wait for mDNS discovery
        client.poll_for_duration(Duration::from_secs(discovery_timeout)).await;
        
        let nodes = client.discovered_nodes();
        
        if nodes.is_empty() {
            println!("{}", formatter.warning("No nodes discovered."));
            println!("{}", formatter.info("Make sure mvp-node is running on the network."));
        } else {
            println!("{}", formatter.success(&format!("Found {} node(s)", nodes.len())));
            println!();
            
            let mut rows = Vec::new();
            
            for conn in nodes {
                let peer_id_short = format!("{}...{}", 
                    &conn.peer_id.to_string()[..8],
                    &conn.peer_id.to_string()[conn.peer_id.to_string().len()-4..]
                );
                
                let (node_id, status, models) = if let Some(ref info) = conn.node_info {
                    (
                        info.node_id.clone(),
                        formatter.status(&info.status),
                        info.models.join(", "),
                    )
                } else {
                    (
                        "unknown".to_string(),
                        formatter.status("discovered"),
                        "-".to_string(),
                    )
                };
                
                let addrs = conn.addresses.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                rows.push(vec![
                    node_id,
                    peer_id_short,
                    status,
                    models,
                    addrs,
                ]);
            }
            
            println!("{}", formatter.table(
                vec!["Node ID", "Peer ID", "Status", "Models", "Address"],
                rows
            ));
        }
        
        if !watch {
            break;
        }
        
        println!("\n{}", formatter.info(&format!("Refreshing every {}s... (Ctrl+C to stop)", interval)));
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
    
    Ok(())
}

/// Find a peer by node ID or peer ID string
async fn find_peer(client: &mut NodeClient, node_id: &str) -> CliResult<libp2p::PeerId> {
    // Wait for discovery
    client.poll_for_duration(Duration::from_secs(2)).await;
    
    // Try to parse as PeerId first
    if let Ok(peer_id) = node_id.parse::<libp2p::PeerId>() {
        if client.find_node_by_peer_id(&peer_id).is_some() {
            return Ok(peer_id);
        }
    }
    
    // Search by node ID
    if let Some((peer_id, _)) = client.find_node_by_id(node_id) {
        return Ok(*peer_id);
    }
    
    // If nothing found, check discovered nodes
    let nodes = client.discovered_nodes();
    if nodes.is_empty() {
        return Err(CliError::NoNodesDiscovered);
    }
    
    // Try partial match on peer ID
    for conn in nodes {
        let peer_str = conn.peer_id.to_string();
        if peer_str.contains(node_id) || node_id.contains(&peer_str[..8]) {
            return Ok(conn.peer_id);
        }
    }
    
    Err(CliError::NotFound(format!("Node '{}' not found", node_id)))
}

/// Get detailed node information
async fn get_node_info(
    client: &mut NodeClient,
    node_id: String,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    let peer_id = find_peer(client, &node_id).await?;
    
    println!("{}", formatter.info(&format!("Querying node {}...", peer_id)));
    
    let info = client.get_node_info(&peer_id).await?;
    
    println!("\n{}", formatter.header(&format!("Node: {}", info.node_id)));
    
    println!("{}", formatter.kv("Node ID", &info.node_id));
    println!("{}", formatter.kv("Peer ID", &info.peer_id));
    println!("{}", formatter.kv("Status", &formatter.status(&info.status)));
    println!("{}", formatter.kv("Version", &info.version));
    println!("{}", formatter.kv("Uptime", &format_uptime(info.uptime_seconds)));
    
    if !info.addresses.is_empty() {
        println!("{}", formatter.kv("Addresses", &info.addresses.join(", ")));
    }
    
    println!("\n{}", formatter.header("Capabilities"));
    for model in &info.models {
        println!("  • {}", model);
    }
    
    println!("\n{}", formatter.header("Statistics"));
    println!("{}", formatter.kv("Connected Peers", &info.peer_count.to_string()));
    println!("{}", formatter.kv("Queue", &format!("{}/{}", info.queue_size, info.queue_capacity)));
    
    Ok(())
}

/// Check node health
async fn check_health(
    client: &mut NodeClient,
    node_id: String,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    let peer_id = find_peer(client, &node_id).await?;
    
    println!("{}", formatter.info(&format!("Checking health of {}...", peer_id)));
    
    let health = client.get_node_health(&peer_id).await?;
    
    println!("\n{}", formatter.header("Health Check"));
    
    let components = [
        ("Network", &health.network),
        ("Model", &health.model),
        ("Queue", &health.queue),
        ("Memory", &health.memory),
        ("CPU", &health.cpu),
    ];
    
    for (name, component) in components {
        let status_str = formatter.status(&component.status);
        let detail = component.message.as_deref().unwrap_or("");
        let usage = component.usage_percent
            .map(|u| format!(" ({:.1}%)", u))
            .unwrap_or_default();
        
        println!("  {} {} {}{}", status_str, name, detail, usage);
    }
    
    println!();
    println!("{}", formatter.kv("Overall", &formatter.status(&health.overall)));
    
    Ok(())
}

/// Show node metrics
async fn show_metrics(
    client: &mut NodeClient,
    node_id: String,
    metric_filter: Option<String>,
    watch: bool,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    loop {
        if watch {
            print!("\x1B[2J\x1B[1;1H");
        }
        
        // Find peer (rediscover on each iteration for watch mode)
        let peer_id = find_peer(client, &node_id).await?;
        
        let metrics = client.get_node_metrics(&peer_id).await?;
        
        println!("{}", formatter.header(&format!("Metrics: {}", node_id)));
        
        let all_rows = vec![
            ("jobs_processed", metrics.jobs_processed.to_string(), "count"),
            ("jobs_completed", metrics.jobs_completed.to_string(), "count"),
            ("jobs_failed", metrics.jobs_failed.to_string(), "count"),
            ("jobs_queued", metrics.jobs_queued.to_string(), "count"),
            ("avg_latency_ms", format!("{:.2}", metrics.avg_inference_latency_ms), "ms"),
            ("tokens_processed", metrics.tokens_processed.to_string(), "count"),
            ("memory_bytes", format_bytes(metrics.memory_usage_bytes), "bytes"),
            ("cpu_usage", format!("{:.1}%", metrics.cpu_usage_percent), "%"),
            ("uptime", format_uptime(metrics.uptime_seconds), "time"),
        ];
        
        let rows: Vec<Vec<String>> = all_rows.into_iter()
            .filter(|(name, _, _)| {
                metric_filter.as_ref()
                    .map(|f| name.contains(f.as_str()))
                    .unwrap_or(true)
            })
            .map(|(name, value, unit)| vec![name.to_string(), value, unit.to_string()])
            .collect();
        
        println!("{}", formatter.table(
            vec!["Metric", "Value", "Unit"],
            rows
        ));
        
        if !watch {
            break;
        }
        
        println!("\n{}", formatter.info("Refreshing every 2s... (Ctrl+C to stop)"));
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    Ok(())
}

/// Ping a node
async fn ping_node(
    client: &mut NodeClient,
    node_id: String,
    count: u32,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    let peer_id = find_peer(client, &node_id).await?;
    
    println!("{}", formatter.header(&format!("Pinging {} ({} times)", peer_id, count)));
    
    let mut rtts = Vec::new();
    
    for i in 0..count {
        match client.ping(&peer_id).await {
            Ok(rtt) => {
                println!("  ping {}: {}ms", i + 1, rtt);
                rtts.push(rtt);
            }
            Err(e) => {
                println!("  ping {}: {} (error)", i + 1, formatter.error(&e.to_string()));
            }
        }
        
        if i < count - 1 {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    
    if !rtts.is_empty() {
        let avg = rtts.iter().sum::<u64>() / rtts.len() as u64;
        let min = *rtts.iter().min().unwrap();
        let max = *rtts.iter().max().unwrap();
        
        println!();
        println!("{}", formatter.success(&format!(
            "min/avg/max = {}/{}ms/{}ms ({}/{} successful)",
            min, avg, max, rtts.len(), count
        )));
    } else {
        println!();
        println!("{}", formatter.error("All pings failed"));
    }
    
    Ok(())
}

/// Format uptime in human-readable format
fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
