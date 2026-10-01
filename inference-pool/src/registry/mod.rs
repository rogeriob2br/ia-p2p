//! Pool Registry module.
//!
//! Manages Pool_Members and their capabilities.
//!
//! ## Responsibilities
//!
//! - Register new members
//! - Track member status (Active, Inactive, Draining)
//! - Store member capabilities and models
//! - Update member metrics
//! - Query members by model support

mod storage;

pub use storage::InMemoryPoolRegistry;

use crate::error::PoolResult;
use crate::models::{MemberMetrics, MemberStatus, ModelInfo, NodeCapabilities, PoolMember};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Member registration data
#[derive(Debug, Clone)]
pub struct MemberRegistration {
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

    /// Registration key used
    pub registration_key: String,
}

impl Default for MemberRegistration {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            peer_id: String::new(),
            capabilities: NodeCapabilities::default(),
            models: Vec::new(),
            lightning_pubkey: String::new(),
            multiaddrs: Vec::new(),
            registration_key: String::new(),
        }
    }
}

/// Pool Registry trait
///
/// Manages the registry of Pool_Members and their capabilities.
#[async_trait]
pub trait PoolRegistry: Send + Sync {
    /// Register a new node as pool member
    ///
    /// Validates the registration and creates a new PoolMember entry.
    /// Returns the created member with assigned member_id.
    async fn register_member(&self, registration: MemberRegistration) -> PoolResult<PoolMember>;

    /// Update member status
    ///
    /// Changes the status of a member (Active, Inactive, Draining, AtRisk).
    async fn update_member_status(&self, node_id: &str, status: MemberStatus) -> PoolResult<()>;

    /// Get all active members
    ///
    /// Returns a list of all members with Active status.
    fn get_active_members(&self) -> Vec<PoolMember>;

    /// Get all members regardless of status
    fn get_all_members(&self) -> Vec<PoolMember>;

    /// Get a specific member by node_id
    fn get_member(&self, node_id: &str) -> Option<PoolMember>;

    /// Get a specific member by member_id
    fn get_member_by_id(&self, member_id: &str) -> Option<PoolMember>;

    /// Get members supporting a specific model
    ///
    /// Returns members that have the specified model in their capabilities.
    fn get_members_for_model(&self, model: &str) -> Vec<PoolMember>;

    /// Remove a member from the pool
    ///
    /// Completely removes the member from the registry.
    async fn remove_member(&self, node_id: &str) -> PoolResult<()>;

    /// Update member metrics
    ///
    /// Updates the performance metrics for a member.
    async fn update_member_metrics(&self, node_id: &str, metrics: MemberMetrics) -> PoolResult<()>;

    /// Update last heartbeat timestamp
    ///
    /// Called when a heartbeat is received from a member.
    async fn update_heartbeat(&self, node_id: &str, timestamp: DateTime<Utc>) -> PoolResult<()>;

    /// Set the Lightning channel ID for a member
    async fn set_channel_id(&self, node_id: &str, channel_id: String) -> PoolResult<()>;

    /// Get count of members by status
    fn count_by_status(&self, status: MemberStatus) -> usize;

    /// Get total member count
    fn total_count(&self) -> usize;

    /// Get aggregated models from all active members
    ///
    /// Returns a deduplicated list of all models available in the pool.
    fn get_available_models(&self) -> Vec<ModelInfo>;

    /// Get model availability info (count of nodes supporting each model)
    fn get_model_availability(&self) -> Vec<(String, u32)>;

    /// Check if a registration key is valid
    fn validate_registration_key(&self, key: &str) -> bool;

    /// Add a valid registration key
    async fn add_registration_key(&self, key: String) -> PoolResult<()>;

    /// Remove a registration key (revoke)
    async fn remove_registration_key(&self, key: &str) -> PoolResult<()>;

    /// Check if node is already registered
    fn is_registered(&self, node_id: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_registration_default() {
        let reg = MemberRegistration::default();
        assert!(reg.node_id.is_empty());
        assert!(reg.models.is_empty());
    }
}
