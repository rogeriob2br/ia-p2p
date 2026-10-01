//! Property tests for the Job Router.
//!
//! Feature: job-routing
//! Tests Properties 9, 10, 11, 12 from the design document.

use chrono::Utc;
use inference_pool::models::{
    InferenceJob, JobType, MemberMetrics, MemberStatus, ModelInfo, NodeCapabilities, PoolMember,
    SlaTier,
};
use inference_pool::registry::{InMemoryPoolRegistry, MemberRegistration};
use inference_pool::router::{
    JobRouter, JobRouterImpl, LeastLoadedStrategy, PerformanceStrategy, RoundRobinStrategy,
    RoutingStrategy,
};
use inference_pool::PoolRegistry;
use proptest::prelude::*;
use std::sync::Arc;

/// Create a test member registration
fn create_registration(
    node_id: &str,
    models: Vec<String>,
) -> MemberRegistration {
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
                model_id: m.clone(),
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

/// Create a test member (for strategy tests only)
fn create_member(
    id: &str,
    models: Vec<String>,
    load: f64,
    latency: f64,
    success: f64,
) -> PoolMember {
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
        models: models
            .iter()
            .map(|m| ModelInfo {
                model_id: m.clone(),
                family: None,
                parameters: None,
                context_length: None,
                quantization: None,
            })
            .collect(),
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

/// Create a test job
fn create_job(id: &str, model: &str) -> InferenceJob {
    InferenceJob {
        job_id: id.to_string(),
        client_id: "client-1".to_string(),
        model: model.to_string(),
        job_type: JobType::ChatCompletion,
        payload: serde_json::json!({}),
        created_at: Utc::now(),
        timeout_ms: 30000,
        sla_tier: SlaTier::Basic,
    }
}

/// Strategy for generating model names
fn model_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("gpt-4".to_string()),
        Just("gpt-3.5-turbo".to_string()),
        Just("llama-3".to_string()),
        Just("mistral-7b".to_string()),
        Just("claude-3".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: job-routing
    /// Property 9: Routing Always Selects Valid Member
    ///
    /// For any job and set of available members supporting the requested model,
    /// the router SHALL return a valid member from the available set.
    ///
    /// Validates: Requirement 5.1
    #[test]
    fn property_9_routing_selects_valid_member(
        member_count in 1usize..10,
        model in model_name_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
                "test-key".to_string(),
            ]));

            // Register members
            let mut member_ids = Vec::new();
            for i in 0..member_count {
                let node_id = format!("node-{}", i);
                let reg = create_registration(&node_id, vec![model.clone()]);
                let member = registry.register_member(reg).await.unwrap();
                member_ids.push(member.member_id.clone());
                registry.update_member_status(&node_id, MemberStatus::Active).await.unwrap();
            }

            let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());
            let job = create_job("job-1", &model);

            let result = router.route_job(&job).await.unwrap();

            // Result must be one of the registered members
            prop_assert!(
                member_ids.contains(&result.member_id),
                "Selected member {} must be in registered set {:?}",
                result.member_id,
                member_ids
            );

            Ok(())
        })?;
    }

    /// Feature: job-routing
    /// Property 10: Routing Only Selects Members With Model
    ///
    /// For any job requesting model M, the router SHALL only select
    /// members that advertise capability for model M.
    ///
    /// Validates: Requirement 5.2
    #[test]
    fn property_10_routing_respects_model_capability(
        target_model in model_name_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
                "test-key".to_string(),
            ]));

            // Register members with different models
            let reg_a = create_registration("node-a", vec![target_model.clone()]);
            let reg_b = create_registration("node-b", vec!["other-model".to_string()]);
            let reg_c = create_registration("node-c", vec![target_model.clone(), "extra-model".to_string()]);

            let member_a = registry.register_member(reg_a).await.unwrap();
            let member_b = registry.register_member(reg_b).await.unwrap();
            let member_c = registry.register_member(reg_c).await.unwrap();

            registry.update_member_status("node-a", MemberStatus::Active).await.unwrap();
            registry.update_member_status("node-b", MemberStatus::Active).await.unwrap();
            registry.update_member_status("node-c", MemberStatus::Active).await.unwrap();

            let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());
            let job = create_job("job-1", &target_model);

            let result = router.route_job(&job).await.unwrap();

            // Result must be member-a or member-c (not member-b)
            prop_assert!(
                result.member_id == member_a.member_id || result.member_id == member_c.member_id,
                "Selected member {} should support model {}",
                result.member_id,
                target_model
            );

            Ok(())
        })?;
    }

    /// Feature: job-routing
    /// Property 11: Round-Robin Distributes Evenly
    ///
    /// For N members and M jobs (where M >> N), round-robin routing
    /// SHALL distribute jobs such that each member gets approximately M/N jobs.
    ///
    /// Validates: Requirement 5.3
    #[test]
    fn property_11_round_robin_even_distribution(
        member_count in 2usize..5,
        jobs_per_member in 5usize..20,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
                "test-key".to_string(),
            ]));
            let model = "gpt-4".to_string();

            // Register members
            let mut member_ids = Vec::new();
            for i in 0..member_count {
                let node_id = format!("node-{}", i);
                let reg = create_registration(&node_id, vec![model.clone()]);
                let member = registry.register_member(reg).await.unwrap();
                member_ids.push(member.member_id.clone());
                registry.update_member_status(&node_id, MemberStatus::Active).await.unwrap();
            }

            let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());
            let total_jobs = member_count * jobs_per_member;

            let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

            for i in 0..total_jobs {
                let job = create_job(&format!("job-{}", i), &model);
                let result = router.route_job(&job).await.unwrap();
                *counts.entry(result.member_id).or_insert(0) += 1;
            }

            // Each member should have exactly jobs_per_member jobs
            for id in &member_ids {
                let count = counts.get(id).copied().unwrap_or(0);
                prop_assert_eq!(
                    count, jobs_per_member,
                    "Member {} should have {} jobs, got {}",
                    id, jobs_per_member, count
                );
            }

            Ok(())
        })?;
    }

    /// Feature: job-routing
    /// Property 12: Least-Loaded Prefers Low Load
    ///
    /// For any set of members with different load levels, the least-loaded
    /// strategy SHALL select the member with the lowest load.
    ///
    /// Validates: Requirement 5.4
    #[test]
    fn property_12_least_loaded_prefers_minimum(
        loads in prop::collection::vec(0.0f64..1.0, 2..10),
    ) {
        let model = "gpt-4".to_string();
        let members: Vec<PoolMember> = loads
            .iter()
            .enumerate()
            .map(|(i, &load)| create_member(&format!("member-{}", i), vec![model.clone()], load, 100.0, 1.0))
            .collect();

        let strategy = LeastLoadedStrategy::new();
        let job = create_job("job-1", &model);

        let result = strategy.select(&members, &job).unwrap();

        // Find the minimum load
        let min_load = loads.iter().cloned().fold(f64::INFINITY, f64::min);
        let selected_idx: usize = result.member_id.strip_prefix("member-").unwrap().parse().unwrap();
        let selected_load = loads[selected_idx];

        prop_assert!(
            (selected_load - min_load).abs() < 0.0001,
            "Selected member has load {}, but minimum is {}",
            selected_load, min_load
        );
    }

    /// Feature: job-routing
    /// Property: Performance Strategy Considers All Factors
    ///
    /// The performance strategy SHALL consider latency, success rate, and load.
    #[test]
    fn performance_strategy_balanced_selection(
        latencies in prop::collection::vec(10.0f64..5000.0, 3..5),
        success_rates in prop::collection::vec(0.5f64..1.0, 3..5),
        loads in prop::collection::vec(0.0f64..1.0, 3..5),
    ) {
        let n = latencies.len().min(success_rates.len()).min(loads.len());
        if n < 2 {
            return Ok(());
        }

        let model = "gpt-4".to_string();
        let members: Vec<PoolMember> = (0..n)
            .map(|i| {
                create_member(
                    &format!("member-{}", i),
                    vec![model.clone()],
                    loads[i],
                    latencies[i],
                    success_rates[i],
                )
            })
            .collect();

        let strategy = PerformanceStrategy::new();
        let job = create_job("job-1", &model);

        let result = strategy.select(&members, &job).unwrap();

        // Just verify a valid member was selected
        prop_assert!(
            result.member_id.starts_with("member-"),
            "Should select a valid member"
        );

        // Verify the reason contains "Performance"
        prop_assert!(
            result.reason.contains("Performance"),
            "Reason should mention performance score"
        );
    }

    /// Feature: job-routing
    /// Property: Router Stats Are Accurate
    ///
    /// After routing N jobs, the router stats SHALL report exactly N total_routed.
    #[test]
    fn router_stats_accuracy(
        job_count in 1usize..50,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
                "test-key".to_string(),
            ]));
            let model = "gpt-4".to_string();

            let reg = create_registration("node-1", vec![model.clone()]);
            registry.register_member(reg).await.unwrap();
            registry.update_member_status("node-1", MemberStatus::Active).await.unwrap();

            let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());

            for i in 0..job_count {
                let job = create_job(&format!("job-{}", i), &model);
                router.route_job(&job).await.unwrap();
            }

            let stats = router.get_stats();

            prop_assert_eq!(
                stats.total_routed as usize, job_count,
                "Stats should show {} routed, got {}",
                job_count, stats.total_routed
            );

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_model_not_found_error() {
        let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
            "test-key".to_string(),
        ]));
        let reg = create_registration("node-1", vec!["gpt-4".to_string()]);
        registry.register_member(reg).await.unwrap();
        registry.update_member_status("node-1", MemberStatus::Active).await.unwrap();

        let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());
        let job = create_job("job-1", "nonexistent-model");

        let result = router.route_job(&job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inactive_member_skipped() {
        let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
            "test-key".to_string(),
        ]));

        let reg = create_registration("node-1", vec!["gpt-4".to_string()]);
        registry.register_member(reg).await.unwrap();
        // Don't activate - stays in Pending status

        let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());
        let job = create_job("job-1", "gpt-4");

        let result = router.route_job(&job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_stats() {
        let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
            "test-key".to_string(),
        ]));
        let reg = create_registration("node-1", vec!["gpt-4".to_string()]);
        registry.register_member(reg).await.unwrap();
        registry.update_member_status("node-1", MemberStatus::Active).await.unwrap();

        let router = JobRouterImpl::new(registry, RoundRobinStrategy::new());

        for i in 0..5 {
            let job = create_job(&format!("job-{}", i), "gpt-4");
            router.route_job(&job).await.unwrap();
        }

        let stats = router.get_stats();
        assert_eq!(stats.total_routed, 5);

        router.reset_stats();

        let stats = router.get_stats();
        assert_eq!(stats.total_routed, 0);
        assert!(stats.jobs_per_member.is_empty());
    }
}
