//! In-memory storage implementation for Pool Registry.

use super::{MemberRegistration, PoolRegistry};
use crate::error::{PoolError, PoolResult};
use crate::models::{MemberMetrics, MemberStatus, ModelInfo, PoolMember};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::RwLock;

/// In-memory implementation of PoolRegistry
///
/// Uses DashMap for concurrent access to members.
#[derive(Debug)]
pub struct InMemoryPoolRegistry {
    /// Members indexed by node_id
    members: DashMap<String, PoolMember>,

    /// Index: member_id -> node_id
    member_id_index: DashMap<String, String>,

    /// Index: model_id -> set of node_ids
    model_index: DashMap<String, HashSet<String>>,

    /// Valid registration keys
    registration_keys: Arc<RwLock<HashSet<String>>>,

    /// Maximum number of members allowed
    max_members: usize,
}

impl InMemoryPoolRegistry {
    /// Create a new in-memory registry
    pub fn new() -> Self {
        Self {
            members: DashMap::new(),
            member_id_index: DashMap::new(),
            model_index: DashMap::new(),
            registration_keys: Arc::new(RwLock::new(HashSet::new())),
            max_members: 1000,
        }
    }

    /// Create a new registry with custom max members
    pub fn with_max_members(max_members: usize) -> Self {
        Self {
            members: DashMap::new(),
            member_id_index: DashMap::new(),
            model_index: DashMap::new(),
            registration_keys: Arc::new(RwLock::new(HashSet::new())),
            max_members,
        }
    }

    /// Create a new registry with pre-loaded registration keys
    pub fn with_registration_keys(keys: Vec<String>) -> Self {
        let registry = Self::new();
        for key in keys {
            registry.registration_keys.write().insert(key);
        }
        registry
    }

    /// Update model index when member is added
    fn update_model_index_add(&self, node_id: &str, models: &[ModelInfo]) {
        for model in models {
            self.model_index
                .entry(model.model_id.clone())
                .or_insert_with(HashSet::new)
                .insert(node_id.to_string());
        }
    }

    /// Update model index when member is removed
    fn update_model_index_remove(&self, node_id: &str, models: &[ModelInfo]) {
        for model in models {
            if let Some(mut entry) = self.model_index.get_mut(&model.model_id) {
                entry.remove(node_id);
                if entry.is_empty() {
                    drop(entry);
                    self.model_index.remove(&model.model_id);
                }
            }
        }
    }
}

impl Default for InMemoryPoolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PoolRegistry for InMemoryPoolRegistry {
    async fn register_member(&self, registration: MemberRegistration) -> PoolResult<PoolMember> {
        // Check if already registered
        if self.members.contains_key(&registration.node_id) {
            return Err(PoolError::invalid_request(format!(
                "Node '{}' is already registered",
                registration.node_id
            )));
        }

        // Check max members
        if self.members.len() >= self.max_members {
            return Err(PoolError::service_unavailable(
                "Pool has reached maximum member capacity",
            ));
        }

        // Validate registration key
        if !self.validate_registration_key(&registration.registration_key) {
            return Err(PoolError::registration("Invalid registration key"));
        }

        // Create member
        let member_id = format!("member-{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let member = PoolMember {
            member_id: member_id.clone(),
            node_id: registration.node_id.clone(),
            peer_id: registration.peer_id,
            capabilities: registration.capabilities,
            models: registration.models.clone(),
            status: MemberStatus::Pending,
            last_heartbeat: now,
            metrics: MemberMetrics::default(),
            lightning_pubkey: registration.lightning_pubkey,
            multiaddrs: registration.multiaddrs,
            registered_at: now,
            channel_id: None,
        };

        // Update indices
        self.member_id_index
            .insert(member_id.clone(), registration.node_id.clone());
        self.update_model_index_add(&registration.node_id, &registration.models);

        // Store member
        self.members.insert(registration.node_id, member.clone());

        tracing::info!(
            member_id = %member.member_id,
            node_id = %member.node_id,
            models = ?member.models.iter().map(|m| &m.model_id).collect::<Vec<_>>(),
            "Registered new pool member"
        );

        Ok(member)
    }

    async fn update_member_status(&self, node_id: &str, status: MemberStatus) -> PoolResult<()> {
        let mut member = self
            .members
            .get_mut(node_id)
            .ok_or_else(|| PoolError::node_not_found(node_id))?;

        let old_status = member.status;
        member.status = status;

        tracing::debug!(
            node_id = %node_id,
            old_status = ?old_status,
            new_status = ?status,
            "Updated member status"
        );

        Ok(())
    }

    fn get_active_members(&self) -> Vec<PoolMember> {
        self.members
            .iter()
            .filter(|entry| entry.value().status == MemberStatus::Active)
            .map(|entry| entry.value().clone())
            .collect()
    }

    fn get_all_members(&self) -> Vec<PoolMember> {
        self.members
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    fn get_member(&self, node_id: &str) -> Option<PoolMember> {
        self.members.get(node_id).map(|entry| entry.value().clone())
    }

    fn get_member_by_id(&self, member_id: &str) -> Option<PoolMember> {
        self.member_id_index
            .get(member_id)
            .and_then(|node_id| self.members.get(node_id.value()).map(|m| m.clone()))
    }

    fn get_members_for_model(&self, model: &str) -> Vec<PoolMember> {
        self.model_index
            .get(model)
            .map(|node_ids| {
                node_ids
                    .iter()
                    .filter_map(|node_id| self.members.get(node_id).map(|m| m.clone()))
                    .filter(|m| m.status == MemberStatus::Active)
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn remove_member(&self, node_id: &str) -> PoolResult<()> {
        let member = self
            .members
            .remove(node_id)
            .ok_or_else(|| PoolError::node_not_found(node_id))?
            .1;

        // Remove from indices
        self.member_id_index.remove(&member.member_id);
        self.update_model_index_remove(node_id, &member.models);

        tracing::info!(
            node_id = %node_id,
            member_id = %member.member_id,
            "Removed pool member"
        );

        Ok(())
    }

    async fn update_member_metrics(&self, node_id: &str, metrics: MemberMetrics) -> PoolResult<()> {
        let mut member = self
            .members
            .get_mut(node_id)
            .ok_or_else(|| PoolError::node_not_found(node_id))?;

        member.metrics = metrics;

        Ok(())
    }

    async fn update_heartbeat(&self, node_id: &str, timestamp: DateTime<Utc>) -> PoolResult<()> {
        let mut member = self
            .members
            .get_mut(node_id)
            .ok_or_else(|| PoolError::node_not_found(node_id))?;

        member.last_heartbeat = timestamp;

        // If member was inactive or pending, mark as active
        if member.status == MemberStatus::Inactive || member.status == MemberStatus::Pending {
            member.status = MemberStatus::Active;
        }

        Ok(())
    }

    async fn set_channel_id(&self, node_id: &str, channel_id: String) -> PoolResult<()> {
        let mut member = self
            .members
            .get_mut(node_id)
            .ok_or_else(|| PoolError::node_not_found(node_id))?;

        member.channel_id = Some(channel_id);

        Ok(())
    }

    fn count_by_status(&self, status: MemberStatus) -> usize {
        self.members
            .iter()
            .filter(|entry| entry.value().status == status)
            .count()
    }

    fn total_count(&self) -> usize {
        self.members.len()
    }

    fn get_available_models(&self) -> Vec<ModelInfo> {
        let mut models: Vec<ModelInfo> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for entry in self.members.iter() {
            if entry.value().status != MemberStatus::Active {
                continue;
            }
            for model in &entry.value().models {
                if !seen.contains(&model.model_id) {
                    seen.insert(model.model_id.clone());
                    models.push(model.clone());
                }
            }
        }

        models
    }

    fn get_model_availability(&self) -> Vec<(String, u32)> {
        self.model_index
            .iter()
            .map(|entry| {
                let active_count = entry
                    .value()
                    .iter()
                    .filter(|node_id| {
                        self.members
                            .get(*node_id)
                            .map(|m| m.status == MemberStatus::Active)
                            .unwrap_or(false)
                    })
                    .count() as u32;
                (entry.key().clone(), active_count)
            })
            .collect()
    }

    fn validate_registration_key(&self, key: &str) -> bool {
        self.registration_keys.read().contains(key)
    }

    async fn add_registration_key(&self, key: String) -> PoolResult<()> {
        self.registration_keys.write().insert(key);
        Ok(())
    }

    async fn remove_registration_key(&self, key: &str) -> PoolResult<()> {
        self.registration_keys.write().remove(key);
        Ok(())
    }

    fn is_registered(&self, node_id: &str) -> bool {
        self.members.contains_key(node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NodeCapabilities;

    fn create_test_registration(node_id: &str, models: Vec<&str>) -> MemberRegistration {
        MemberRegistration {
            node_id: node_id.to_string(),
            peer_id: format!("peer-{}", node_id),
            capabilities: NodeCapabilities {
                gpu_memory_gb: 24,
                gpu_count: 1,
                gpu_model: Some("RTX 4090".to_string()),
                cpu_cores: 16,
                ram_gb: 64,
                max_concurrent: 4,
                max_context_length: 8192,
                features: vec!["chat".to_string()],
            },
            models: models
                .into_iter()
                .map(|m| ModelInfo {
                    model_id: m.to_string(),
                    family: None,
                    parameters: None,
                    context_length: None,
                    quantization: None,
                })
                .collect(),
            lightning_pubkey: format!("02{}", node_id),
            multiaddrs: vec![format!("/ip4/127.0.0.1/tcp/9000")],
            registration_key: "test-key".to_string(),
        }
    }

    #[tokio::test]
    async fn test_register_member() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["test-key".to_string()]);

        let registration = create_test_registration("node-1", vec!["llama-7b"]);
        let member = registry.register_member(registration).await.unwrap();

        assert!(!member.member_id.is_empty());
        assert_eq!(member.node_id, "node-1");
        assert_eq!(member.status, MemberStatus::Pending);
        assert_eq!(member.models.len(), 1);
    }

    #[tokio::test]
    async fn test_duplicate_registration_fails() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["test-key".to_string()]);

        let registration = create_test_registration("node-1", vec!["llama-7b"]);
        registry.register_member(registration.clone()).await.unwrap();

        let result = registry.register_member(registration).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_key_fails() {
        let registry = InMemoryPoolRegistry::new(); // No keys added

        let registration = create_test_registration("node-1", vec!["llama-7b"]);
        let result = registry.register_member(registration).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_members_for_model() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["test-key".to_string()]);

        // Register two nodes with llama-7b
        let reg1 = create_test_registration("node-1", vec!["llama-7b", "mistral-7b"]);
        let reg2 = create_test_registration("node-2", vec!["llama-7b"]);
        let reg3 = create_test_registration("node-3", vec!["mistral-7b"]);

        let m1 = registry.register_member(reg1).await.unwrap();
        let m2 = registry.register_member(reg2).await.unwrap();
        let m3 = registry.register_member(reg3).await.unwrap();

        // Activate all members
        registry.update_member_status(&m1.node_id, MemberStatus::Active).await.unwrap();
        registry.update_member_status(&m2.node_id, MemberStatus::Active).await.unwrap();
        registry.update_member_status(&m3.node_id, MemberStatus::Active).await.unwrap();

        // Query by model
        let llama_members = registry.get_members_for_model("llama-7b");
        assert_eq!(llama_members.len(), 2);

        let mistral_members = registry.get_members_for_model("mistral-7b");
        assert_eq!(mistral_members.len(), 2);

        let unknown_members = registry.get_members_for_model("unknown-model");
        assert_eq!(unknown_members.len(), 0);
    }

    #[tokio::test]
    async fn test_update_heartbeat_activates_member() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["test-key".to_string()]);

        let registration = create_test_registration("node-1", vec!["llama-7b"]);
        registry.register_member(registration).await.unwrap();

        // Member starts as Pending
        let member = registry.get_member("node-1").unwrap();
        assert_eq!(member.status, MemberStatus::Pending);

        // Heartbeat should activate
        registry.update_heartbeat("node-1", Utc::now()).await.unwrap();

        let member = registry.get_member("node-1").unwrap();
        assert_eq!(member.status, MemberStatus::Active);
    }

    #[tokio::test]
    async fn test_remove_member() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["test-key".to_string()]);

        let registration = create_test_registration("node-1", vec!["llama-7b"]);
        registry.register_member(registration).await.unwrap();

        assert!(registry.is_registered("node-1"));

        registry.remove_member("node-1").await.unwrap();

        assert!(!registry.is_registered("node-1"));
        assert!(registry.get_members_for_model("llama-7b").is_empty());
    }

    #[tokio::test]
    async fn test_model_availability() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["test-key".to_string()]);

        let reg1 = create_test_registration("node-1", vec!["llama-7b"]);
        let reg2 = create_test_registration("node-2", vec!["llama-7b"]);

        let m1 = registry.register_member(reg1).await.unwrap();
        let m2 = registry.register_member(reg2).await.unwrap();

        registry.update_member_status(&m1.node_id, MemberStatus::Active).await.unwrap();
        registry.update_member_status(&m2.node_id, MemberStatus::Active).await.unwrap();

        let availability = registry.get_model_availability();
        assert_eq!(availability.len(), 1);
        assert_eq!(availability[0].0, "llama-7b");
        assert_eq!(availability[0].1, 2);
    }
}







