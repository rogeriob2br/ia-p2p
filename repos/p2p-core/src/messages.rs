//! Message types for P2P communication

use serde::{Deserialize, Serialize};

/// Topics for gossipsub communication
pub mod topics {
    pub const ANNOUNCE: &str = "p2p-ai/announce/1.0";
    pub const JOB_OFFER: &str = "p2p-ai/job-offer/1.0";
    pub const RECEIPT: &str = "p2p-ai/receipt/1.0";
}

/// Announce message: node broadcasts its capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceMsg {
    pub manifest_id: String,
    pub version: String,
    pub manifest_hash: String,
    pub capabilities: Capabilities,
    pub payment_preferences: Vec<String>,
    pub timestamp: i64,
    pub peer_id: String,
}

/// Hardware capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub hostname: String,
    pub total_memory_gb: f64,
    pub cpus: Vec<String>,
    pub gpus: Vec<GpuInfo>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub driver_version: Option<String>,
    pub memory_gb: Option<f64>,
}

/// Job offer message: relay broadcasts available work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOfferMsg {
    pub job_id: String,
    pub model: String,
    pub mode: JobMode,
    pub reward: f64,
    pub currency: String,
    pub requirements: Requirements,
    pub policy_refs: Vec<String>,
    pub timestamp: i64,
    pub relay_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum JobMode {
    #[default]
    Batch,
    Session,
}

/// Job requirements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Requirements {
    pub min_vram_gb: Option<f64>,
    pub min_ram_gb: Option<f64>,
    pub gpu_vendor: Option<String>,
}

/// Job claim: agent claims a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobClaimMsg {
    pub job_id: String,
    pub agent_peer_id: String,
    pub manifest_id: String,
    pub timestamp: i64,
}

/// Receipt message: agent reports job completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMsg {
    pub job_id: String,
    pub agent_peer_id: String,
    pub manifest_id: String,
    pub status: JobStatus,
    pub output_hash: Option<String>,
    pub metrics: ExecutionMetrics,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetrics {
    pub duration_ms: u64,
    pub tokens_processed: Option<u64>,
    pub peak_memory_mb: Option<u64>,
}

/// Wrapper enum for all gossipsub messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum P2PMessage {
    Announce(AnnounceMsg),
    JobOffer(JobOfferMsg),
    JobClaim(JobClaimMsg),
    Receipt(ReceiptMsg),
}

impl P2PMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}












