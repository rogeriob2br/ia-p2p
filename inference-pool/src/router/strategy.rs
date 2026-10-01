//! Routing strategies for job distribution.

use super::RoutingResult;
use crate::error::{PoolError, PoolResult};
use crate::models::{InferenceJob, PoolMember};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Routing strategy trait
pub trait RoutingStrategy: Send + Sync {
    /// Select a member for the given job
    fn select(&self, members: &[PoolMember], job: &InferenceJob) -> PoolResult<RoutingResult>;

    /// Strategy name
    fn name(&self) -> &str;
}

/// Round-robin routing strategy
#[derive(Debug)]
pub struct RoundRobinStrategy {
    /// Current index
    index: AtomicUsize,
}

impl RoundRobinStrategy {
    /// Create a new round-robin strategy
    pub fn new() -> Self {
        Self {
            index: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingStrategy for RoundRobinStrategy {
    fn select(&self, members: &[PoolMember], _job: &InferenceJob) -> PoolResult<RoutingResult> {
        if members.is_empty() {
            return Err(PoolError::service_unavailable("No members available"));
        }

        let idx = self.index.fetch_add(1, Ordering::Relaxed) % members.len();
        let member = &members[idx];

        Ok(RoutingResult {
            member_id: member.member_id.clone(),
            member_addr: member.multiaddrs.first().cloned().unwrap_or_default(),
            reason: format!("Round-robin selection (index {})", idx),
        })
    }

    fn name(&self) -> &str {
        "round-robin"
    }
}

/// Least-loaded routing strategy
#[derive(Debug, Default)]
pub struct LeastLoadedStrategy;

impl LeastLoadedStrategy {
    /// Create a new least-loaded strategy
    pub fn new() -> Self {
        Self
    }
}

impl RoutingStrategy for LeastLoadedStrategy {
    fn select(&self, members: &[PoolMember], _job: &InferenceJob) -> PoolResult<RoutingResult> {
        if members.is_empty() {
            return Err(PoolError::service_unavailable("No members available"));
        }

        // Select member with lowest current load
        let member = members
            .iter()
            .min_by(|a, b| {
                a.metrics
                    .current_load
                    .partial_cmp(&b.metrics.current_load)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        Ok(RoutingResult {
            member_id: member.member_id.clone(),
            member_addr: member.multiaddrs.first().cloned().unwrap_or_default(),
            reason: format!("Least loaded ({:.1}% load)", member.metrics.current_load * 100.0),
        })
    }

    fn name(&self) -> &str {
        "least-loaded"
    }
}

/// Performance-based routing strategy
///
/// Considers:
/// - Average latency
/// - Success rate
/// - Current load
#[derive(Debug)]
pub struct PerformanceStrategy {
    /// Weight for latency (lower is better)
    latency_weight: f64,

    /// Weight for success rate (higher is better)
    success_weight: f64,

    /// Weight for load (lower is better)
    load_weight: f64,
}

impl PerformanceStrategy {
    /// Create a new performance strategy with default weights
    pub fn new() -> Self {
        Self {
            latency_weight: 0.3,
            success_weight: 0.5,
            load_weight: 0.2,
        }
    }

    /// Create with custom weights
    pub fn with_weights(latency: f64, success: f64, load: f64) -> Self {
        let total = latency + success + load;
        Self {
            latency_weight: latency / total,
            success_weight: success / total,
            load_weight: load / total,
        }
    }

    /// Calculate score for a member (higher is better)
    fn score(&self, member: &PoolMember) -> f64 {
        let metrics = &member.metrics;

        // Normalize latency (assume max 10000ms)
        let latency_score = 1.0 - (metrics.avg_latency_ms / 10000.0).min(1.0);

        // Success rate already 0-1
        let success_score = metrics.success_rate;

        // Load already 0-1, lower is better
        let load_score = 1.0 - metrics.current_load;

        // Weighted sum
        latency_score * self.latency_weight
            + success_score * self.success_weight
            + load_score * self.load_weight
    }
}

impl Default for PerformanceStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingStrategy for PerformanceStrategy {
    fn select(&self, members: &[PoolMember], _job: &InferenceJob) -> PoolResult<RoutingResult> {
        if members.is_empty() {
            return Err(PoolError::service_unavailable("No members available"));
        }

        // Score and select best member
        let (member, score) = members
            .iter()
            .map(|m| (m, self.score(m)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        Ok(RoutingResult {
            member_id: member.member_id.clone(),
            member_addr: member.multiaddrs.first().cloned().unwrap_or_default(),
            reason: format!("Performance score: {:.3}", score),
        })
    }

    fn name(&self) -> &str {
        "performance"
    }
}

/// Affinity-based routing strategy
///
/// Routes to members in the same context group when possible
#[derive(Debug, Default)]
pub struct AffinityStrategy {
    /// Fallback strategy
    fallback: LeastLoadedStrategy,
}

impl AffinityStrategy {
    /// Create a new affinity strategy
    pub fn new() -> Self {
        Self::default()
    }
}

impl RoutingStrategy for AffinityStrategy {
    fn select(&self, members: &[PoolMember], job: &InferenceJob) -> PoolResult<RoutingResult> {
        if members.is_empty() {
            return Err(PoolError::service_unavailable("No members available"));
        }

        // If job has context affinity, prefer that member
        if let Some(context_id) = job.payload.get("context_id").and_then(|v| v.as_str()) {
            // Look for member with matching context
            // (In real implementation, we'd check context group membership)
            let prefix = format!("ctx-{}", &context_id[..8.min(context_id.len())]);

            if let Some(member) = members
                .iter()
                .find(|m| m.member_id.contains(&prefix) || m.peer_id.contains(&prefix))
            {
                return Ok(RoutingResult {
                    member_id: member.member_id.clone(),
                    member_addr: member.multiaddrs.first().cloned().unwrap_or_default(),
                    reason: format!("Context affinity: {}", context_id),
                });
            }
        }

        // Fallback to least-loaded
        self.fallback.select(members, job)
    }

    fn name(&self) -> &str {
        "affinity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{JobType, MemberMetrics, MemberStatus, ModelInfo, NodeCapabilities, SlaTier};
    use chrono::Utc;

    fn create_test_member(id: &str, load: f64, latency: f64, success: f64) -> PoolMember {
        PoolMember {
            member_id: id.to_string(),
            node_id: format!("node-{}", id),
            peer_id: format!("12D3KooW{}", id),
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
            models: vec![ModelInfo {
                model_id: "gpt-4".to_string(),
                family: None,
                parameters: None,
                context_length: None,
                quantization: None,
            }],
            status: MemberStatus::Active,
            last_heartbeat: Utc::now(),
            metrics: MemberMetrics {
                avg_latency_ms: latency,
                success_rate: success,
                current_load: load,
                jobs_completed: 100,
                jobs_failed: 0,
                tokens_processed: 10000,
                consensus_participation: 1.0,
                uptime_percent: 99.9,
            },
            lightning_pubkey: "02abc...".to_string(),
            multiaddrs: vec![format!("/ip4/127.0.0.1/tcp/8000/{}", id)],
            registered_at: Utc::now(),
            channel_id: None,
        }
    }

    fn create_test_job() -> InferenceJob {
        InferenceJob {
            job_id: "job-1".to_string(),
            client_id: "client-1".to_string(),
            model: "gpt-4".to_string(),
            job_type: JobType::ChatCompletion,
            payload: serde_json::json!({}),
            created_at: Utc::now(),
            timeout_ms: 30000,
            sla_tier: SlaTier::Basic,
        }
    }

    #[test]
    fn test_round_robin_distribution() {
        let strategy = RoundRobinStrategy::new();
        let members = vec![
            create_test_member("a", 0.0, 100.0, 1.0),
            create_test_member("b", 0.0, 100.0, 1.0),
            create_test_member("c", 0.0, 100.0, 1.0),
        ];
        let job = create_test_job();

        let mut counts = std::collections::HashMap::new();
        for _ in 0..9 {
            let result = strategy.select(&members, &job).unwrap();
            *counts.entry(result.member_id).or_insert(0) += 1;
        }

        // Each should be selected 3 times
        assert_eq!(counts.get("a"), Some(&3));
        assert_eq!(counts.get("b"), Some(&3));
        assert_eq!(counts.get("c"), Some(&3));
    }

    #[test]
    fn test_least_loaded_selects_minimum() {
        let strategy = LeastLoadedStrategy::new();
        let members = vec![
            create_test_member("a", 0.8, 100.0, 1.0),  // 80% load
            create_test_member("b", 0.2, 100.0, 1.0),  // 20% load (lowest)
            create_test_member("c", 0.5, 100.0, 1.0),  // 50% load
        ];
        let job = create_test_job();

        let result = strategy.select(&members, &job).unwrap();
        assert_eq!(result.member_id, "b");
    }

    #[test]
    fn test_performance_considers_all_factors() {
        let strategy = PerformanceStrategy::new();

        // Member a: low latency, perfect success, high load
        // Member b: medium latency, high success, low load (best overall)
        // Member c: high latency, poor success, low load
        let members = vec![
            create_test_member("a", 0.9, 100.0, 1.0),   // High load
            create_test_member("b", 0.1, 500.0, 0.95), // Best overall
            create_test_member("c", 0.1, 2000.0, 0.50), // Poor success
        ];
        let job = create_test_job();

        let result = strategy.select(&members, &job).unwrap();
        // b should win due to balanced factors
        assert_eq!(result.member_id, "b");
    }

    #[test]
    fn test_empty_members_returns_error() {
        let strategies: Vec<Box<dyn RoutingStrategy>> = vec![
            Box::new(RoundRobinStrategy::new()),
            Box::new(LeastLoadedStrategy::new()),
            Box::new(PerformanceStrategy::new()),
            Box::new(AffinityStrategy::new()),
        ];
        let job = create_test_job();

        for strategy in strategies {
            let result = strategy.select(&[], &job);
            assert!(result.is_err());
        }
    }
}
