//! relay-hub: Job distribution hub for the P2P AI network
//!
//! Provides HTTP API for job submission and gossipsub distribution.

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use libp2p::identity::Keypair;
use p2p_core::{
    messages::{topics, JobMode, JobOfferMsg, P2PMessage, ReceiptMsg, Requirements},
    network::{NetworkConfig, NetworkEvent, NetworkNode},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use uuid::Uuid;

const CONFIG_DIR: &str = ".p2p-ai";
const DB_FILE: &str = "relay.db";

#[derive(Parser, Debug)]
#[command(name = "relay-hub")]
#[command(about = "Job relay hub for decentralized AI network")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the relay hub
    Run {
        /// HTTP API port
        #[arg(long, default_value = "3000")]
        http_port: u16,

        /// P2P listen port
        #[arg(long, default_value = "4001")]
        p2p_port: u16,
    },
    /// Submit a job via CLI
    Submit {
        /// Job JSON file
        #[arg(long)]
        job: PathBuf,
    },
}

/// Job submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobRequest {
    model: String,
    #[serde(default)]
    mode: String,
    reward: f64,
    currency: String,
    #[serde(default)]
    requirements: RequirementsRequest,
    #[serde(default)]
    policy_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RequirementsRequest {
    min_vram_gb: Option<f64>,
    min_ram_gb: Option<f64>,
    gpu_vendor: Option<String>,
}

/// Job response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobResponse {
    job_id: String,
    status: String,
}

/// Shared application state
struct AppState {
    db: Mutex<Connection>,
    network_tx: mpsc::Sender<(String, P2PMessage)>,
    local_peer_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("relay_hub=info".parse()?)
                .add_directive("p2p_core=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { http_port, p2p_port } => {
            run_relay(http_port, p2p_port).await?;
        }
        Commands::Submit { job } => {
            submit_job_cli(&job).await?;
        }
    }

    Ok(())
}

async fn run_relay(http_port: u16, p2p_port: u16) -> Result<()> {
    // Initialize database
    let db_path = dirs::home_dir()
        .expect("home directory")
        .join(CONFIG_DIR)
        .join(DB_FILE);
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let db = Connection::open(&db_path)?;
    init_db(&db)?;
    info!("Database initialized at {:?}", db_path);

    // Generate or load keypair
    let keypair = Keypair::generate_ed25519();
    let local_peer_id = libp2p::PeerId::from(keypair.public()).to_string();
    info!("Relay peer ID: {}", local_peer_id);

    // Setup channels
    let (event_tx, mut event_rx) = mpsc::channel::<NetworkEvent>(256);
    let (network_tx, network_rx) = mpsc::channel::<(String, P2PMessage)>(256);

    // Create network node
    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse()?;
    let config = NetworkConfig {
        listen_addr,
        bootstrap_peers: Vec::new(),
    };

    let mut network = NetworkNode::new(keypair, config, event_tx).await?;
    network.subscribe_all()?;

    let network_peer_id = network.local_peer_id().to_string();

    // Shared state
    let state = Arc::new(AppState {
        db: Mutex::new(db),
        network_tx: network_tx.clone(),
        local_peer_id: network_peer_id.clone(),
    });

    // HTTP API
    let app = Router::new()
        .route("/", get(health))
        .route("/health", get(health))
        .route("/jobs", post(submit_job))
        .route("/jobs", get(list_jobs))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    // Spawn HTTP server
    let http_addr = format!("0.0.0.0:{}", http_port);
    info!("HTTP API listening on http://{}", http_addr);
    let listener = tokio::net::TcpListener::bind(&http_addr).await?;
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Spawn event handler
    let state_for_events = state.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                NetworkEvent::MessageReceived { topic, message, source } => {
                    info!("Received message on {}: {:?} from {:?}", topic, message, source);
                    if let P2PMessage::Receipt(receipt) = message {
                        let db = state_for_events.db.lock().await;
                        if let Err(e) = update_job_receipt(&db, &receipt) {
                            error!("Failed to update job receipt: {}", e);
                        }
                    }
                }
                NetworkEvent::PeerDiscovered(peer) => {
                    info!("Peer discovered: {}", peer);
                }
                NetworkEvent::PeerExpired(peer) => {
                    info!("Peer expired: {}", peer);
                }
                _ => {}
            }
        }
    });

    // Run network with publisher channel
    info!("Relay hub running. Press Ctrl+C to stop.");
    network.run_with_publisher(network_rx).await;

    Ok(())
}

fn init_db(db: &Connection) -> Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS jobs (
            job_id TEXT PRIMARY KEY,
            model TEXT NOT NULL,
            mode TEXT NOT NULL,
            reward REAL NOT NULL,
            currency TEXT NOT NULL,
            requirements TEXT,
            policy_refs TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            agent_peer_id TEXT,
            receipt TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn submit_job(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JobRequest>,
) -> Result<Json<JobResponse>, StatusCode> {
    let job_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mode = if req.mode.is_empty() { "batch" } else { &req.mode };

    // Insert into database
    {
        let db = state.db.lock().await;
        db.execute(
            "INSERT INTO jobs (job_id, model, mode, reward, currency, requirements, policy_refs, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, ?8)",
            params![
                &job_id,
                &req.model,
                mode,
                req.reward,
                &req.currency,
                serde_json::to_string(&req.requirements).unwrap_or_default(),
                serde_json::to_string(&req.policy_refs).unwrap_or_default(),
                &now,
            ],
        )
        .map_err(|e| {
            error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // Create job offer message
    let job_offer = JobOfferMsg {
        job_id: job_id.clone(),
        model: req.model,
        mode: if mode == "session" {
            JobMode::Session
        } else {
            JobMode::Batch
        },
        reward: req.reward,
        currency: req.currency,
        requirements: Requirements {
            min_vram_gb: req.requirements.min_vram_gb,
            min_ram_gb: req.requirements.min_ram_gb,
            gpu_vendor: req.requirements.gpu_vendor,
        },
        policy_refs: req.policy_refs,
        timestamp: chrono::Utc::now().timestamp(),
        relay_peer_id: state.local_peer_id.clone(),
    };

    // Queue for network broadcast
    let msg = P2PMessage::JobOffer(job_offer);
    state
        .network_tx
        .send((topics::JOB_OFFER.to_string(), msg))
        .await
        .map_err(|e| {
            error!("Failed to queue message: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Job submitted: {}", job_id);

    Ok(Json(JobResponse {
        job_id,
        status: "pending".to_string(),
    }))
}

#[derive(Debug, Serialize)]
struct JobListItem {
    job_id: String,
    model: String,
    status: String,
    reward: f64,
    currency: String,
    created_at: String,
}

async fn list_jobs(State(state): State<Arc<AppState>>) -> Result<Json<Vec<JobListItem>>, StatusCode> {
    let db = state.db.lock().await;
    let mut stmt = db
        .prepare("SELECT job_id, model, status, reward, currency, created_at FROM jobs ORDER BY created_at DESC LIMIT 100")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jobs = stmt
        .query_map([], |row| {
            Ok(JobListItem {
                job_id: row.get(0)?,
                model: row.get(1)?,
                status: row.get(2)?,
                reward: row.get(3)?,
                currency: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(jobs))
}

fn update_job_receipt(db: &Connection, receipt: &ReceiptMsg) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let status = match receipt.status {
        p2p_core::messages::JobStatus::Completed => "completed",
        p2p_core::messages::JobStatus::Failed => "failed",
        p2p_core::messages::JobStatus::Timeout => "timeout",
    };

    db.execute(
        "UPDATE jobs SET status = ?1, agent_peer_id = ?2, receipt = ?3, updated_at = ?4 WHERE job_id = ?5",
        params![
            status,
            &receipt.agent_peer_id,
            serde_json::to_string(receipt).unwrap_or_default(),
            &now,
            &receipt.job_id,
        ],
    )?;

    info!("Updated job {} with status {}", receipt.job_id, status);
    Ok(())
}

async fn submit_job_cli(job_path: &PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(job_path)?;
    let job: JobRequest = serde_json::from_str(&content)?;

    let client = reqwest::Client::new();
    let resp = client
        .post("http://127.0.0.1:3000/jobs")
        .json(&job)
        .send()
        .await?;

    if resp.status().is_success() {
        let result: JobResponse = resp.json().await?;
        println!("Job submitted: {}", result.job_id);
    } else {
        eprintln!("Failed to submit job: {}", resp.status());
    }

    Ok(())
}

