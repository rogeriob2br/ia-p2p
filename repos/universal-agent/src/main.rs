//! universal-agent: Executor node for the decentralized AI network
//!
//! Announces capabilities, listens for jobs, executes them, and publishes receipts.

use anyhow::Result;
use clap::{Parser, Subcommand};
use libp2p::identity::Keypair;
use p2p_core::{
    messages::{
        topics, AnnounceMsg, Capabilities, ExecutionMetrics, GpuInfo, JobOfferMsg, JobStatus,
        P2PMessage, ReceiptMsg,
    },
    network::{keypair_to_hex, NetworkConfig, NetworkEvent, NetworkNode},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::sync::mpsc;
use tracing::info;

const CONFIG_DIR_NAME: &str = ".p2p-ai";
const KEYS_FILE: &str = "keys.json";
const MANIFEST_FILE: &str = "manifest.local.json";
const JOBS_DIR: &str = "jobs";

#[derive(Parser, Debug)]
#[command(name = "universal-agent")]
#[command(about = "Universal agent for decentralized AI network")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Detect local hardware capabilities
    Detect,

    /// Initialize keys and local manifest
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,

        /// Import existing secret key (hex)
        #[arg(long)]
        secret_key: Option<String>,
    },

    /// Run the agent: announce + listen for jobs
    Run {
        /// Bootstrap peer multiaddr (e.g., /ip4/127.0.0.1/tcp/4001/p2p/<peer_id>)
        #[arg(long)]
        bootstrap: Option<String>,

        /// P2P listen port
        #[arg(long, default_value = "0")]
        port: u16,

        /// Announce interval in seconds
        #[arg(long, default_value = "30")]
        announce_interval: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HwInventory {
    hostname: String,
    total_memory_gb: f64,
    cpus: Vec<String>,
    gpus: Vec<GpuInfoLocal>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct GpuInfoLocal {
    name: String,
    vendor: String,
    driver_version: Option<String>,
    memory_gb: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyBundle {
    public_key: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LocalManifest {
    manifest_id: String,
    version: String,
    capabilities: HwInventory,
    payment_preferences: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("universal_agent=info".parse()?)
                .add_directive("p2p_core=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Detect => {
            let inv = detect_hw();
            println!("{}", serde_json::to_string_pretty(&inv)?);
        }
        Commands::Init { force, secret_key } => {
            init_agent(force, secret_key)?;
            println!("Initialized config at {:?}", config_dir());
        }
        Commands::Run {
            bootstrap,
            port,
            announce_interval,
        } => {
            run_agent(bootstrap, port, announce_interval).await?;
        }
    }

    Ok(())
}

fn detect_hw() -> HwInventory {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let total_memory_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

    let cpus: Vec<String> = sys
        .cpus()
        .iter()
        .take(1) // Just get first CPU brand
        .map(|cpu| cpu.brand().to_string())
        .collect();

    let gpus = detect_gpus();

    HwInventory {
        hostname,
        total_memory_gb,
        cpus,
        gpus,
    }
}

#[cfg(feature = "nvidia")]
fn detect_gpus() -> Vec<GpuInfoLocal> {
    use nvml_wrapper::Nvml;

    let mut gpus = Vec::new();

    if let Ok(nvml) = Nvml::init() {
        if let Ok(count) = nvml.device_count() {
            for i in 0..count {
                if let Ok(device) = nvml.device_by_index(i) {
                    let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
                    let memory_gb = device
                        .memory_info()
                        .map(|m| m.total as f64 / (1024.0 * 1024.0 * 1024.0))
                        .ok();
                    let driver = nvml.sys_driver_version().ok();

                    gpus.push(GpuInfoLocal {
                        name,
                        vendor: "NVIDIA".to_string(),
                        driver_version: driver,
                        memory_gb,
                    });
                }
            }
        }
    }

    gpus
}

#[cfg(not(feature = "nvidia"))]
fn detect_gpus() -> Vec<GpuInfoLocal> {
    Vec::new()
}

fn init_agent(force: bool, secret_key: Option<String>) -> Result<()> {
    let dir = config_dir();
    if dir.exists() && !force {
        anyhow::bail!(
            "Config directory {:?} already exists. Use --force to overwrite.",
            dir
        );
    }
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join(JOBS_DIR))?;

    let keypair = if let Some(sk_hex) = secret_key {
        let bytes = hex::decode(&sk_hex)?;
        Keypair::ed25519_from_bytes(bytes)?
    } else {
        Keypair::generate_ed25519()
    };

    let (public_hex, secret_hex) = keypair_to_hex(&keypair)?;

    let key_bundle = KeyBundle {
        public_key: public_hex.clone(),
        secret_key: secret_hex,
    };
    let keys_path = dir.join(KEYS_FILE);
    write_json(&keys_path, &key_bundle)?;

    let manifest = LocalManifest {
        manifest_id: format!("manifest-{}", &public_hex[..8]),
        version: "0.1.0".to_string(),
        capabilities: detect_hw(),
        payment_preferences: vec!["lightning".into()],
    };
    let manifest_path = dir.join(MANIFEST_FILE);
    write_json(&manifest_path, &manifest)?;

    info!("Generated keypair. Public key: {}", public_hex);

    Ok(())
}

async fn run_agent(bootstrap: Option<String>, port: u16, announce_interval: u64) -> Result<()> {
    // Load keys and manifest
    let keys_path = config_dir().join(KEYS_FILE);
    let manifest_path = config_dir().join(MANIFEST_FILE);

    if !keys_path.exists() || !manifest_path.exists() {
        anyhow::bail!("Agent not initialized. Run 'universal-agent init' first.");
    }

    let key_bundle: KeyBundle = read_json(&keys_path)?;
    let manifest: LocalManifest = read_json(&manifest_path)?;

    let secret_bytes = hex::decode(&key_bundle.secret_key)?;
    let keypair = Keypair::ed25519_from_bytes(secret_bytes)?;

    info!("Loaded manifest: {}", manifest.manifest_id);

    // Setup network
    let (event_tx, mut event_rx) = mpsc::channel::<NetworkEvent>(256);

    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
    let bootstrap_peers = if let Some(addr) = bootstrap {
        vec![addr.parse()?]
    } else {
        Vec::new()
    };

    let config = NetworkConfig {
        listen_addr,
        bootstrap_peers,
    };

    let mut network = NetworkNode::new(keypair, config, event_tx).await?;
    let local_peer_id = network.local_peer_id().to_string();
    info!("Agent peer ID: {}", local_peer_id);

    // Subscribe to topics
    network.subscribe(topics::JOB_OFFER)?;
    network.subscribe(topics::ANNOUNCE)?;

    // Channel for publishing messages
    let (pub_tx, pub_rx) = mpsc::channel::<(String, P2PMessage)>(64);

    // Create announce message
    let manifest_hash = compute_manifest_hash(&manifest);
    let announce_msg = create_announce_msg(&manifest, &manifest_hash, &local_peer_id);

    // Spawn announce loop
    let pub_tx_announce = pub_tx.clone();
    let announce_msg_clone = announce_msg.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(announce_interval));
        loop {
            interval.tick().await;
            let msg = P2PMessage::Announce(announce_msg_clone.clone());
            if pub_tx_announce
                .send((topics::ANNOUNCE.to_string(), msg))
                .await
                .is_err()
            {
                break;
            }
            info!("Announced capabilities");
        }
    });

    // Spawn event handler
    let pub_tx_events = pub_tx.clone();
    let manifest_for_handler = manifest.clone();
    let peer_id_for_handler = local_peer_id.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                NetworkEvent::MessageReceived {
                    topic,
                    message,
                    source,
                } => {
                    info!("Received on {}: {:?} from {:?}", topic, message, source);

                    if let P2PMessage::JobOffer(job) = message {
                        // Check if we can handle this job
                        if should_accept_job(&job, &manifest_for_handler) {
                            info!("Accepting job: {}", job.job_id);

                            // Execute mock job
                            let receipt = execute_mock_job(&job, &manifest_for_handler, &peer_id_for_handler).await;

                            // Publish receipt
                            let msg = P2PMessage::Receipt(receipt);
                            let _ = pub_tx_events
                                .send((topics::RECEIPT.to_string(), msg))
                                .await;
                        } else {
                            info!("Skipping job {}: requirements not met", job.job_id);
                        }
                    }
                }
                NetworkEvent::PeerDiscovered(peer) => {
                    info!("Discovered peer: {}", peer);
                }
                NetworkEvent::PeerExpired(peer) => {
                    info!("Peer expired: {}", peer);
                }
                _ => {}
            }
        }
    });

    // Main loop: run network with publisher channel
    info!("Agent running. Press Ctrl+C to stop.");

    network.run_with_publisher(pub_rx).await;

    Ok(())
}

fn should_accept_job(job: &JobOfferMsg, manifest: &LocalManifest) -> bool {
    // Check VRAM requirements
    if let Some(min_vram) = job.requirements.min_vram_gb {
        let total_vram: f64 = manifest
            .capabilities
            .gpus
            .iter()
            .filter_map(|g| g.memory_gb)
            .sum();
        if total_vram < min_vram {
            return false;
        }
    }

    // Check RAM requirements
    if let Some(min_ram) = job.requirements.min_ram_gb {
        if manifest.capabilities.total_memory_gb < min_ram {
            return false;
        }
    }

    // Check GPU vendor
    if let Some(ref vendor) = job.requirements.gpu_vendor {
        let has_vendor = manifest
            .capabilities
            .gpus
            .iter()
            .any(|g| g.vendor.to_lowercase() == vendor.to_lowercase());
        if !has_vendor {
            return false;
        }
    }

    true
}

async fn execute_mock_job(
    job: &JobOfferMsg,
    manifest: &LocalManifest,
    peer_id: &str,
) -> ReceiptMsg {
    let start = Instant::now();

    // Create job directory
    let job_dir = config_dir().join(JOBS_DIR).join(&job.job_id);
    let _ = fs::create_dir_all(&job_dir);

    // Write job info
    let job_file = job_dir.join("job.json");
    let _ = fs::write(&job_file, serde_json::to_string_pretty(job).unwrap_or_default());

    info!("Executing mock job: {}", job.job_id);

    // Simulate work
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Write mock output
    let output = format!(
        "Mock execution completed for model: {}\nTimestamp: {}",
        job.model,
        chrono::Utc::now()
    );
    let output_file = job_dir.join("output.txt");
    let _ = fs::write(&output_file, &output);

    let duration = start.elapsed();

    // Create receipt
    let output_hash = {
        let mut hasher = Sha256::new();
        hasher.update(output.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    ReceiptMsg {
        job_id: job.job_id.clone(),
        agent_peer_id: peer_id.to_string(),
        manifest_id: manifest.manifest_id.clone(),
        status: JobStatus::Completed,
        output_hash: Some(output_hash),
        metrics: ExecutionMetrics {
            duration_ms: duration.as_millis() as u64,
            tokens_processed: Some(100), // Mock
            peak_memory_mb: Some(512),   // Mock
        },
        timestamp: chrono::Utc::now().timestamp(),
    }
}

fn create_announce_msg(manifest: &LocalManifest, hash: &str, peer_id: &str) -> AnnounceMsg {
    AnnounceMsg {
        manifest_id: manifest.manifest_id.clone(),
        version: manifest.version.clone(),
        manifest_hash: hash.to_string(),
        capabilities: Capabilities {
            hostname: manifest.capabilities.hostname.clone(),
            total_memory_gb: manifest.capabilities.total_memory_gb,
            cpus: manifest.capabilities.cpus.clone(),
            gpus: manifest
                .capabilities
                .gpus
                .iter()
                .map(|g| GpuInfo {
                    name: g.name.clone(),
                    vendor: g.vendor.clone(),
                    driver_version: g.driver_version.clone(),
                    memory_gb: g.memory_gb,
                })
                .collect(),
        },
        payment_preferences: manifest.payment_preferences.clone(),
        timestamp: chrono::Utc::now().timestamp(),
        peer_id: peer_id.to_string(),
    }
}

fn compute_manifest_hash(manifest: &LocalManifest) -> String {
    let serialized = serde_json::to_vec(manifest).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    format!("{:x}", hasher.finalize())
}

fn write_json(path: &PathBuf, value: &impl Serialize) -> Result<()> {
    let data = serde_json::to_string_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<T> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("home directory not found")
        .join(CONFIG_DIR_NAME)
}
