//! Job Router module.
//!
//! Routes inference requests to available pool members based on
//! capabilities, load, and performance.
//!
//! ## Routing Strategies
//!
//! - Round-robin with capability filtering
//! - Least-loaded node selection
//! - Performance-based selection (latency, success rate)
//! - Affinity-based routing (context groups)

mod strategy;

pub use strategy::{
    LeastLoadedStrategy, PerformanceStrategy, RoundRobinStrategy, RoutingStrategy,
};

use crate::error::{PoolError, PoolResult};
use crate::models::{InferenceJob, MemberStatus};
use crate::registry::PoolRegistry;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// Job routing result
#[derive(Debug, Clone)]
pub struct RoutingResult {
    /// Selected member ID
    pub member_id: String,

    /// Member P2P address
    pub member_addr: String,

    /// Routing reason
    pub reason: String,
}

/// Job Router trait
#[async_trait]
pub trait JobRouter: Send + Sync {
    /// Route a job to an available member
    async fn route_job(&self, job: &InferenceJob) -> PoolResult<RoutingResult>;

    /// Get routing statistics
    fn get_stats(&self) -> RouterStats;

    /// Reset statistics
    fn reset_stats(&self);
}

/// Router statistics
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    /// Total jobs routed
    pub total_routed: u64,

    /// Successful routings
    pub successful: u64,

    /// Failed routings (no available member)
    pub failed: u64,

    /// Jobs per member
    pub jobs_per_member: std::collections::HashMap<String, u64>,
}

/// Job Router implementation
pub struct JobRouterImpl<R: PoolRegistry, S: RoutingStrategy> {
    /// Pool registry
    registry: Arc<R>,

    /// Routing strategy
    strategy: S,

    /// Pending jobs (job_id -> member_id)
    pending_jobs: DashMap<String, String>,

    /// Statistics
    stats: DashMap<String, u64>,

    /// Total routed counter
    total_routed: std::sync::atomic::AtomicU64,

    /// Failed counter
    failed: std::sync::atomic::AtomicU64,
}

impl<R: PoolRegistry, S: RoutingStrategy> JobRouterImpl<R, S> {
    /// Create a new job router
    pub fn new(registry: Arc<R>, strategy: S) -> Self {
        Self {
            registry,
            strategy,
            pending_jobs: DashMap::new(),
            stats: DashMap::new(),
            total_routed: std::sync::atomic::AtomicU64::new(0),
            failed: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Get the registry reference
    pub fn registry(&self) -> &Arc<R> {
        &self.registry
    }

    /// Mark a job as complete
    pub fn complete_job(&self, job_id: &str) -> Option<String> {
        self.pending_jobs.remove(job_id).map(|(_, member_id)| member_id)
    }

    /// Get pending job count for a member
    pub fn pending_count(&self, member_id: &str) -> usize {
        self.pending_jobs
            .iter()
            .filter(|entry| entry.value() == member_id)
            .count()
    }
}

#[async_trait]
impl<R: PoolRegistry + 'static, S: RoutingStrategy + 'static> JobRouter for JobRouterImpl<R, S> {
    async fn route_job(&self, job: &InferenceJob) -> PoolResult<RoutingResult> {
        // Get members that support the requested model
        let members = self.registry.get_members_for_model(&job.model);

        if members.is_empty() {
            self.failed
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(PoolError::model_not_found(&job.model));
        }

        // Filter to active members only
        let active_members: Vec<_> = members
            .into_iter()
            .filter(|m| m.status == MemberStatus::Active)
            .collect();

        if active_members.is_empty() {
            self.failed
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(PoolError::service_unavailable(
                "No active nodes available for this model",
            ));
        }

        // Apply routing strategy
        let selected = self.strategy.select(&active_members, job)?;

        // Record the pending job
        self.pending_jobs
            .insert(job.job_id.clone(), selected.member_id.clone());

        // Update stats
        self.total_routed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .entry(selected.member_id.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        tracing::debug!(
            job_id = %job.job_id,
            model = %job.model,
            member_id = %selected.member_id,
            reason = %selected.reason,
            "Routed job to member"
        );

        Ok(selected)
    }

    fn get_stats(&self) -> RouterStats {
        let jobs_per_member: std::collections::HashMap<String, u64> = self
            .stats
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();

        RouterStats {
            total_routed: self.total_routed.load(std::sync::atomic::Ordering::Relaxed),
            successful: self.total_routed.load(std::sync::atomic::Ordering::Relaxed)
                - self.failed.load(std::sync::atomic::Ordering::Relaxed),
            failed: self.failed.load(std::sync::atomic::Ordering::Relaxed),
            jobs_per_member,
        }
    }

    fn reset_stats(&self) {
        self.stats.clear();
        self.total_routed
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.failed.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{JobType, ModelInfo, NodeCapabilities, SlaTier};
    use crate::registry::{InMemoryPoolRegistry, MemberRegistration};
    use chrono::Utc;

    fn create_test_registration(node_id: &str, models: Vec<&str>) -> MemberRegistration {
        MemberRegistration {
            node_id: node_id.to_string(),
            peer_id: format!("12D3KooW{}", node_id),
            capabilities: NodeCapabilities {
                gpu_memory_gb: 24,
                gpu_count: 1,
                gpu_model: None,
                cpu_cores: 8,
                ram_gb: 32,
                max_concurrent: 4,
                max_context_length: 4096,
                features: vec![],
            },
            models: models
                .iter()
                .map(|m| ModelInfo {
                    model_id: m.to_string(),
                    family: None,
                    parameters: None,
                    context_length: None,
                    quantization: None,
                })
                .collect(),
            lightning_pubkey: "02abc...".to_string(),
            multiaddrs: vec![format!("/ip4/127.0.0.1/tcp/8000/{}", node_id)],
            registration_key: "test-key".to_string(),
        }
    }

    #[tokio::test]
    async fn test_route_job_success() {
        let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
            "test-key".to_string(),
        ]));

        // Register and activate a member
        let reg = create_test_registration("node-1", vec!["gpt-4", "gpt-3.5"]);
        let member = registry.register_member(reg).await.unwrap();
        registry
            .update_member_status(&member.node_id, MemberStatus::Active)
            .await
            .unwrap();

        let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());

        let job = InferenceJob {
            job_id: "job-1".to_string(),
            client_id: "client-1".to_string(),
            model: "gpt-4".to_string(),
            job_type: JobType::ChatCompletion,
            payload: serde_json::json!({}),
            created_at: Utc::now(),
            timeout_ms: 30000,
            sla_tier: SlaTier::Basic,
        };

        let result = router.route_job(&job).await.unwrap();
        assert!(result.member_id.starts_with("member-"));
    }

    #[tokio::test]
    async fn test_route_job_no_members() {
        let registry = Arc::new(InMemoryPoolRegistry::new());
        let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());

        let job = InferenceJob {
            job_id: "job-1".to_string(),
            client_id: "client-1".to_string(),
            model: "gpt-4".to_string(),
            job_type: JobType::ChatCompletion,
            payload: serde_json::json!({}),
            created_at: Utc::now(),
            timeout_ms: 30000,
            sla_tier: SlaTier::Basic,
        };

        let result = router.route_job(&job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
            "test-key".to_string(),
        ]));
        let reg = create_test_registration("node-1", vec!["gpt-4"]);
        let member = registry.register_member(reg).await.unwrap();
        registry
            .update_member_status(&member.node_id, MemberStatus::Active)
            .await
            .unwrap();

        let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());

        // Route some jobs
        for i in 0..5 {
            let job = InferenceJob {
                job_id: format!("job-{}", i),
                client_id: "client-1".to_string(),
                model: "gpt-4".to_string(),
                job_type: JobType::ChatCompletion,
                payload: serde_json::json!({}),
                created_at: Utc::now(),
                timeout_ms: 30000,
                sla_tier: SlaTier::Basic,
            };
            router.route_job(&job).await.unwrap();
        }

        let stats = router.get_stats();
        assert_eq!(stats.total_routed, 5);
    }
}
