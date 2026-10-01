//! Pool-specific data models.
//!
//! Types for pool membership, registration, status, and billing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Pool Member
// ============================================================================

/// Pool member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMember {
    /// Unique member ID assigned by pool
    pub member_id: String,

    /// Node's own ID
    pub node_id: String,

    /// libp2p peer ID
    pub peer_id: String,

    /// Node capabilities
    pub capabilities: NodeCapabilities,

    /// Supported models
    pub models: Vec<ModelInfo>,

    /// Current status
    pub status: MemberStatus,

    /// Last heartbeat received
    pub last_heartbeat: DateTime<Utc>,

    /// Performance metrics
    pub metrics: MemberMetrics,

    /// Lightning Network public key
    pub lightning_pubkey: String,

    /// P2P multiaddresses
    pub multiaddrs: Vec<String>,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,

    /// Associated Lightning channel ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
}

/// Node capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeCapabilities {
    /// GPU memory in GB
    pub gpu_memory_gb: u64,

    /// Number of GPUs
    pub gpu_count: u32,

    /// GPU model/type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_model: Option<String>,

    /// CPU cores
    pub cpu_cores: u32,

    /// RAM in GB
    pub ram_gb: u64,

    /// Maximum concurrent requests
    pub max_concurrent: u32,

    /// Maximum context length
    pub max_context_length: u32,

    /// Supported features
    #[serde(default)]
    pub features: Vec<String>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    /// Model identifier
    pub model_id: String,

    /// Model family (e.g., "llama", "mistral")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,

    /// Parameter count (e.g., "7B", "70B")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<String>,

    /// Maximum context length for this model
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,

    /// Quantization (e.g., "Q4_K_M", "FP16")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
}

/// Member status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemberStatus {
    /// Active and accepting jobs
    Active,
    /// Not responding to heartbeats
    Inactive,
    /// Draining (no new jobs, finishing existing)
    Draining,
    /// At risk of deregistration
    AtRisk,
    /// Pending verification
    Pending,
}

/// Member performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberMetrics {
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,

    /// Current load (0.0 - 1.0)
    pub current_load: f64,

    /// Total jobs completed
    pub jobs_completed: u64,

    /// Total jobs failed
    pub jobs_failed: u64,

    /// Total tokens processed
    pub tokens_processed: u64,

    /// Consensus participation rate
    #[serde(default)]
    pub consensus_participation: f64,

    /// Uptime percentage
    #[serde(default)]
    pub uptime_percent: f64,
}

// ============================================================================
// Registration
// ============================================================================

/// Node registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolRegistrationRequest {
    /// Registration key provided by pool operator
    pub registration_key: String,

    /// Node's own ID
    pub node_id: String,

    /// libp2p peer ID
    pub peer_id: String,

    /// Node capabilities
    pub capabilities: NodeCapabilities,

    /// Supported models
    pub models: Vec<ModelInfo>,

    /// Lightning Network public key
    pub lightning_pubkey: String,

    /// P2P multiaddresses
    pub multiaddrs: Vec<String>,
}

/// Registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    /// Registration status
    pub status: RegistrationStatus,

    /// Assigned member ID (if accepted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<String>,

    /// Lightning channel ID (if opened)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    /// Pool information
    pub pool_info: PoolInfo,

    /// Optional message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Registration status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RegistrationStatus {
    /// Registration accepted
    Accepted,
    /// Registration rejected
    Rejected,
    /// Pending verification
    Pending,
}

// ============================================================================
// Pool Status and Info
// ============================================================================

/// Pool status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatus {
    /// Pool identifier
    pub pool_id: String,

    /// Number of active members
    pub active_members: u32,

    /// Total capacity
    pub total_capacity: u32,

    /// Available models
    pub models_available: Vec<String>,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Requests per minute
    pub requests_per_minute: f64,

    /// Uptime percentage
    pub uptime_percentage: f64,

    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Pool info response (public, for marketing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    /// Pool identifier
    pub pool_id: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// Operator name
    pub operator: String,

    /// Fee structure
    pub fee_structure: FeeStructure,

    /// Available SLA tiers
    pub sla_tiers: Vec<SlaTierInfo>,

    /// Pool policies
    pub policies: PoolPolicies,

    /// Registration endpoint
    pub registration_endpoint: String,

    /// Pool version
    pub version: String,
}

/// Fee structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    /// Base fee per 1000 tokens
    pub base_fee_per_1k_tokens: f64,

    /// Currency
    pub currency: String,

    /// Tier multipliers
    pub tier_multipliers: HashMap<String, f64>,
}

/// SLA tier information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaTierInfo {
    /// Tier identifier
    pub tier: SlaTier,

    /// Maximum latency guarantee (ms)
    pub max_latency_ms: u64,

    /// Availability SLA (percentage)
    pub availability_sla: f64,

    /// Priority level
    pub priority: u32,

    /// Price multiplier
    pub price_multiplier: f64,
}

/// SLA tier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SlaTier {
    /// Free tier
    Free,
    /// Basic tier
    Basic,
    /// Professional tier
    Pro,
    /// Enterprise tier
    Enterprise,
}

impl Default for SlaTier {
    fn default() -> Self {
        Self::Free
    }
}

/// Pool policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolPolicies {
    /// Maximum request size in bytes
    pub max_request_size_bytes: u64,

    /// Maximum tokens per request
    pub max_tokens_per_request: u64,

    /// Supported models
    pub supported_models: Vec<String>,

    /// Rate limits by tier
    pub rate_limits: HashMap<String, RateLimit>,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per minute
    pub requests_per_minute: u32,

    /// Tokens per day
    pub tokens_per_day: u64,
}

// ============================================================================
// Heartbeat
// ============================================================================

/// Heartbeat from node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    /// Node ID
    pub node_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Current load (0.0 - 1.0)
    pub current_load: f64,

    /// Available capacity
    pub available_capacity: u32,

    /// Node status
    pub status: NodeStatus,
}

/// Node status in heartbeat
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Active and accepting jobs
    Active,
    /// Busy, at capacity
    Busy,
    /// Draining, not accepting new jobs
    Draining,
}

// ============================================================================
// Jobs
// ============================================================================

/// Inference job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceJob {
    /// Unique job ID
    pub job_id: String,

    /// Client ID
    pub client_id: String,

    /// Model to use
    pub model: String,

    /// Job type
    pub job_type: JobType,

    /// Request payload
    pub payload: serde_json::Value,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Timeout in milliseconds
    pub timeout_ms: u64,

    /// Client's SLA tier
    pub sla_tier: SlaTier,
}

/// Job type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Chat completion
    ChatCompletion,
    /// Embedding
    Embedding,
    /// Completion (legacy)
    Completion,
}

/// Inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Job ID
    pub job_id: String,

    /// Success status
    pub success: bool,

    /// Response payload
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,

    /// Error message if failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Processing latency in milliseconds
    pub latency_ms: u64,

    /// Tokens used
    pub tokens_used: u64,

    /// Member that processed the job
    pub member_id: String,
}

// ============================================================================
// Default implementations
// ============================================================================

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            base_fee_per_1k_tokens: 0.001,
            currency: "USD".to_string(),
            tier_multipliers: HashMap::from([
                ("free".to_string(), 1.0),
                ("basic".to_string(), 1.0),
                ("pro".to_string(), 0.9),
                ("enterprise".to_string(), 0.8),
            ]),
        }
    }
}

impl Default for PoolPolicies {
    fn default() -> Self {
        Self {
            max_request_size_bytes: 10 * 1024 * 1024, // 10MB
            max_tokens_per_request: 128_000,
            supported_models: vec![],
            rate_limits: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_status_serialization() {
        assert_eq!(
            serde_json::to_string(&MemberStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&MemberStatus::Inactive).unwrap(),
            "\"inactive\""
        );
    }

    #[test]
    fn test_registration_request_serialization() {
        let request = PoolRegistrationRequest {
            registration_key: "test-key".to_string(),
            node_id: "node-1".to_string(),
            peer_id: "12D3KooW...".to_string(),
            capabilities: NodeCapabilities {
                gpu_memory_gb: 24,
                gpu_count: 1,
                gpu_model: Some("RTX 4090".to_string()),
                cpu_cores: 16,
                ram_gb: 64,
                max_concurrent: 4,
                max_context_length: 8192,
                features: vec!["chat".to_string(), "embeddings".to_string()],
            },
            models: vec![ModelInfo {
                model_id: "llama-7b".to_string(),
                family: Some("llama".to_string()),
                parameters: Some("7B".to_string()),
                context_length: Some(4096),
                quantization: Some("Q4_K_M".to_string()),
            }],
            lightning_pubkey: "02abc...".to_string(),
            multiaddrs: vec!["/ip4/1.2.3.4/tcp/9000".to_string()],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"registration_key\":\"test-key\""));
        assert!(json.contains("\"gpu_memory_gb\":24"));

        let parsed: PoolRegistrationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.node_id, "node-1");
        assert_eq!(parsed.capabilities.gpu_memory_gb, 24);
    }

    #[test]
    fn test_sla_tier_serialization() {
        assert_eq!(
            serde_json::to_string(&SlaTier::Enterprise).unwrap(),
            "\"enterprise\""
        );

        let tier: SlaTier = serde_json::from_str("\"pro\"").unwrap();
        assert_eq!(tier, SlaTier::Pro);
    }

    #[test]
    fn test_job_type_serialization() {
        assert_eq!(
            serde_json::to_string(&JobType::ChatCompletion).unwrap(),
            "\"chat_completion\""
        );
        assert_eq!(
            serde_json::to_string(&JobType::Embedding).unwrap(),
            "\"embedding\""
        );
    }
}







