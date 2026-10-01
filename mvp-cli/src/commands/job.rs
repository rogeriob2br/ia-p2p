//! Job command handlers
//!
//! Commands for submitting and managing inference jobs via real P2P.
//!
//! Implements Requirements 5.1, 5.3, 5.4, 5.5

use clap::Subcommand;
use std::path::PathBuf;
use std::time::Duration;
use crate::config::CliConfig;
use crate::output::OutputFormatter;
use crate::error::{CliResult, CliError};
use crate::client::NodeClient;
use tracing::info;

#[derive(Subcommand, Debug)]
pub enum JobCommand {
    /// Submit a new inference job
    Submit {
        /// Input text for inference
        input: String,
        
        /// Node to submit job to
        #[arg(short, long)]
        node: Option<String>,
        
        /// Read input from file
        #[arg(short = 'f', long)]
        input_file: Option<PathBuf>,
        
        /// Model to use for inference
        #[arg(short, long)]
        model: Option<String>,
        
        /// Wait for job completion
        #[arg(short, long)]
        wait: bool,
        
        /// Timeout in seconds (when waiting)
        #[arg(long, default_value = "60")]
        timeout: u64,
    },
    
    /// Check status of a job
    Status {
        /// Job ID to check
        job_id: String,
        
        /// Node where job was submitted
        #[arg(short, long)]
        node: Option<String>,
    },
    
    /// List all jobs
    List {
        /// Node to query
        #[arg(short, long)]
        node: Option<String>,
        
        /// Filter by status (pending, processing, completed, failed)
        #[arg(short, long)]
        status: Option<String>,
        
        /// Maximum number of jobs to show
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    
    /// Get result of a completed job
    Result {
        /// Job ID to get result for
        job_id: String,
        
        /// Node where job was submitted
        #[arg(short, long)]
        node: Option<String>,
    },
    
    /// Cancel a pending or running job
    Cancel {
        /// Job ID to cancel
        job_id: String,
        
        /// Node where job was submitted
        #[arg(short, long)]
        node: Option<String>,
    },
}

/// Execute a job command
pub async fn execute(
    cmd: JobCommand,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    // Create P2P client
    let mut client = NodeClient::new(config.timeout_secs).await?;
    
    // Start listening
    let addr = client.start().await?;
    info!("CLI started on {}", addr);
    
    match cmd {
        JobCommand::Submit { input, node, input_file, model, wait, timeout } => {
            submit_job(&mut client, input, node, input_file, model, wait, timeout, config, formatter).await
        }
        JobCommand::Status { job_id, node } => {
            get_status(&mut client, job_id, node, formatter).await
        }
        JobCommand::List { node, status, limit } => {
            list_jobs(&mut client, node, status, limit, formatter).await
        }
        JobCommand::Result { job_id, node } => {
            get_result(&mut client, job_id, node, formatter).await
        }
        JobCommand::Cancel { job_id, node } => {
            cancel_job(&mut client, job_id, node, formatter).await
        }
    }
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
        if peer_str.contains(node_id) || node_id.contains(&peer_str[..8.min(peer_str.len())]) {
            return Ok(conn.peer_id);
        }
    }
    
    Err(CliError::NotFound(format!("Node '{}' not found", node_id)))
}

/// Get first available node
async fn find_any_peer(client: &mut NodeClient) -> CliResult<libp2p::PeerId> {
    client.poll_for_duration(Duration::from_secs(2)).await;
    
    let nodes = client.discovered_nodes();
    if nodes.is_empty() {
        return Err(CliError::NoNodesDiscovered);
    }
    
    Ok(nodes[0].peer_id)
}

async fn submit_job(
    client: &mut NodeClient,
    input: String,
    node: Option<String>,
    input_file: Option<PathBuf>,
    model: Option<String>,
    wait: bool,
    timeout_secs: u64,
    config: &CliConfig,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    // Get input text
    let input_text = if let Some(file_path) = input_file {
        std::fs::read_to_string(&file_path)
            .map_err(|e| CliError::IoError(e))?
    } else {
        input
    };
    
    // Get model
    let model_name = model.or(config.default_model.clone())
        .unwrap_or_else(|| "tinyllama".to_string());
    
    println!("{}", formatter.info("Discovering nodes..."));
    
    // Find the target node
    let peer_id = if let Some(ref node_id) = node {
        find_peer(client, node_id).await?
    } else {
        find_any_peer(client).await?
    };
    
    println!("{}", formatter.info(&format!("Submitting job to {}", peer_id)));
    println!("{}", formatter.kv("Model", &model_name));
    println!("{}", formatter.kv("Input", &truncate(&input_text, 100)));
    
    // Submit job via P2P
    let response = client.submit_job(&peer_id, &model_name, &input_text).await?;
    
    println!();
    println!("{}", formatter.success(&format!("Job submitted: {}", response.job_id)));
    println!("{}", formatter.kv("Node", &response.node_id));
    println!("{}", formatter.kv("Status", &response.status));
    
    if let Some(pos) = response.queue_position {
        println!("{}", formatter.kv("Queue Position", &pos.to_string()));
    }
    
    if wait {
        println!();
        println!("{}", formatter.info("Waiting for completion..."));
        
        let start = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_secs);
        
        loop {
            // Poll for a bit
            client.poll_for_duration(Duration::from_secs(2)).await;
            
            // Check job status
            match client.get_job_status(&peer_id, &response.job_id).await {
                Ok(status) => {
                    if status.status == "completed" || status.status == "failed" {
                        println!();
                        println!("{}", formatter.header("Result"));
                        println!("{}", formatter.kv("Job ID", &status.job_id));
                        println!("{}", formatter.kv("Status", &formatter.status(&status.status)));
                        
                        if status.status == "completed" {
                            // Try to get the result
                            if let Ok(result) = client.get_job_result(&peer_id, &response.job_id).await {
                                if let Some(output) = result.output {
                                    println!();
                                    println!("Output:");
                                    println!("  {}", output);
                                }
                            }
                        } else if let Some(error) = status.error {
                            println!("{}", formatter.kv("Error", &error));
                        }
                        
                        break;
                    }
                    
                    // Show progress if available
                    if let Some(progress) = status.progress {
                        print!("\r{}", formatter.info(&format!("Progress: {:.1}%", progress)));
                    }
                }
                Err(e) => {
                    // Job might not be tracked yet, continue polling
                    if start.elapsed() > Duration::from_secs(5) {
                        println!("{}", formatter.warning(&format!("Could not get status: {}", e)));
                    }
                }
            }
            
            if start.elapsed() > timeout_duration {
                println!();
                println!("{}", formatter.warning(&format!("Timeout after {}s", timeout_secs)));
                println!("{}", formatter.info(&format!("Job {} is still running. Use 'job status {}' to check later.", 
                    response.job_id, response.job_id)));
                break;
            }
        }
    }
    
    Ok(())
}

async fn get_status(
    client: &mut NodeClient,
    job_id: String,
    node: Option<String>,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    // Find the target node
    let peer_id = if let Some(ref node_id) = node {
        find_peer(client, node_id).await?
    } else {
        find_any_peer(client).await?
    };
    
    println!("{}", formatter.info(&format!("Querying job status from {}...", peer_id)));
    
    let status = client.get_job_status(&peer_id, &job_id).await?;
    
    println!();
    println!("{}", formatter.header("Job Status"));
    
    let rows = vec![vec![
        status.job_id.clone(),
        formatter.status(&status.status),
        status.node_id.clone().unwrap_or_else(|| "-".to_string()),
        format_timestamp(status.submitted_at),
    ]];
    
    println!("{}", formatter.table(
        vec!["Job ID", "Status", "Node", "Submitted"],
        rows
    ));
    
    if let Some(progress) = status.progress {
        println!("{}", formatter.kv("Progress", &format!("{:.1}%", progress)));
    }
    
    if let Some(error) = status.error {
        println!("{}", formatter.kv("Error", &error));
    }
    
    Ok(())
}

async fn list_jobs(
    client: &mut NodeClient,
    node: Option<String>,
    _status_filter: Option<String>,
    _limit: usize,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    // Find the target node
    let peer_id = if let Some(ref node_id) = node {
        find_peer(client, node_id).await?
    } else {
        find_any_peer(client).await?
    };
    
    println!("{}", formatter.info(&format!("Querying jobs from {}...", peer_id)));
    
    let jobs = client.list_jobs(&peer_id).await?;
    
    println!();
    println!("{}", formatter.header(&format!("Jobs ({} total)", jobs.len())));
    
    if jobs.is_empty() {
        println!("{}", formatter.info("No jobs found."));
    } else {
        let rows: Vec<Vec<String>> = jobs.iter()
            .map(|job| vec![
                job.job_id.clone(),
                formatter.status(&job.status),
                job.model.clone(),
                job.node_id.clone().unwrap_or_else(|| "-".to_string()),
                format_timestamp(job.submitted_at),
            ])
            .collect();
        
        println!("{}", formatter.table(
            vec!["Job ID", "Status", "Model", "Node", "Submitted"],
            rows
        ));
    }
    
    Ok(())
}

async fn get_result(
    client: &mut NodeClient,
    job_id: String,
    node: Option<String>,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    // Find the target node
    let peer_id = if let Some(ref node_id) = node {
        find_peer(client, node_id).await?
    } else {
        find_any_peer(client).await?
    };
    
    println!("{}", formatter.info(&format!("Querying job result from {}...", peer_id)));
    
    let result = client.get_job_result(&peer_id, &job_id).await?;
    
    println!();
    println!("{}", formatter.header(&format!("Result for {}", job_id)));
    
    println!("{}", formatter.kv("Job ID", &result.job_id));
    println!("{}", formatter.kv("Status", &formatter.status(&result.status)));
    println!("{}", formatter.kv("Duration", &format!("{}ms", result.metrics.duration_ms)));
    
    if let Some(tokens) = result.metrics.tokens_processed {
        println!("{}", formatter.kv("Tokens Processed", &tokens.to_string()));
    }
    
    if let Some(ref error) = result.error {
        println!();
        println!("{}", formatter.error(&format!("Error: {}", error)));
    }
    
    if let Some(ref output) = result.output {
        println!();
        println!("Output:");
        println!("  {}", output);
    }
    
    Ok(())
}

async fn cancel_job(
    client: &mut NodeClient,
    job_id: String,
    node: Option<String>,
    formatter: &OutputFormatter,
) -> CliResult<()> {
    println!("{}", formatter.info("Discovering nodes..."));
    
    // Find the target node
    let peer_id = if let Some(ref node_id) = node {
        find_peer(client, node_id).await?
    } else {
        find_any_peer(client).await?
    };
    
    println!("{}", formatter.info(&format!("Sending cancel request to {}...", peer_id)));
    
    let success = client.cancel_job(&peer_id, &job_id).await?;
    
    if success {
        println!("{}", formatter.success(&format!("Job {} cancelled", job_id)));
    } else {
        println!("{}", formatter.warning(&format!("Could not cancel job {}", job_id)));
    }
    
    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Utc};
    
    if timestamp == 0 {
        return "-".to_string();
    }
    
    DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "-".to_string())
}
