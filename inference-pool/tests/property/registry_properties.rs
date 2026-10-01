//! Property tests for Pool Registry.
//!
//! Feature: api-integration
//! Tests Property 1 from the design document.

use inference_pool::models::{MemberMetrics, MemberStatus, ModelInfo, NodeCapabilities};
use inference_pool::registry::{InMemoryPoolRegistry, MemberRegistration, PoolRegistry};
use proptest::prelude::*;

/// Strategy for generating random node IDs
fn node_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9]{8,32}")
        .unwrap()
        .prop_filter("node_id not empty", |s| !s.is_empty())
}

/// Strategy for generating random model IDs
fn model_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9-]{3,30}")
        .unwrap()
        .prop_filter("model_id not empty", |s| !s.is_empty())
}

/// Strategy for generating ModelInfo
fn model_info_strategy() -> impl Strategy<Value = ModelInfo> {
    (
        model_id_strategy(),
        prop::option::of(prop::string::string_regex("[a-z]{3,10}").unwrap()),
        prop::option::of(prop::string::string_regex("[0-9]+B").unwrap()),
        prop::option::of(1024u32..131072u32),
        prop::option::of(prop::string::string_regex("Q[0-9]_K_[A-Z]").unwrap()),
    )
        .prop_map(|(model_id, family, parameters, context_length, quantization)| {
            ModelInfo {
                model_id,
                family,
                parameters,
                context_length,
                quantization,
            }
        })
}

/// Strategy for generating NodeCapabilities
fn capabilities_strategy() -> impl Strategy<Value = NodeCapabilities> {
    (
        1u64..256u64,      // gpu_memory_gb
        1u32..8u32,        // gpu_count
        prop::option::of(prop::string::string_regex("RTX [0-9]{4}").unwrap()),
        1u32..128u32,      // cpu_cores
        8u64..512u64,      // ram_gb
        1u32..32u32,       // max_concurrent
        1024u32..131072u32, // max_context_length
    )
        .prop_map(
            |(gpu_memory_gb, gpu_count, gpu_model, cpu_cores, ram_gb, max_concurrent, max_context_length)| {
                NodeCapabilities {
                    gpu_memory_gb,
                    gpu_count,
                    gpu_model,
                    cpu_cores,
                    ram_gb,
                    max_concurrent,
                    max_context_length,
                    features: vec!["chat".to_string(), "embeddings".to_string()],
                }
            },
        )
}

/// Strategy for generating MemberRegistration
fn registration_strategy() -> impl Strategy<Value = MemberRegistration> {
    (
        node_id_strategy(),
        prop::string::string_regex("12D3KooW[a-zA-Z0-9]{40}").unwrap(),
        capabilities_strategy(),
        prop::collection::vec(model_info_strategy(), 1..5),
        prop::string::string_regex("02[a-f0-9]{64}").unwrap(),
    )
        .prop_map(|(node_id, peer_id, capabilities, models, lightning_pubkey)| {
            MemberRegistration {
                node_id: node_id.clone(),
                peer_id,
                capabilities,
                models,
                lightning_pubkey,
                multiaddrs: vec![format!("/ip4/127.0.0.1/tcp/9000")],
                registration_key: "valid-key".to_string(),
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: api-integration
    /// Property 1: Node Registration Round-Trip
    ///
    /// For any valid node registration with capabilities (models, GPU, memory),
    /// registering the node and then querying the registry SHALL return
    /// the same capabilities that were registered.
    ///
    /// Validates: Requirements 1.1, 1.2
    #[test]
    fn property_1_node_registration_round_trip(
        registration in registration_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create registry with valid key
            let registry = InMemoryPoolRegistry::with_registration_keys(vec!["valid-key".to_string()]);

            // Register the member
            let result = registry.register_member(registration.clone()).await;
            prop_assert!(result.is_ok(), "Registration should succeed");

            let member = result.unwrap();

            // Verify member_id was assigned
            prop_assert!(!member.member_id.is_empty());
            prop_assert!(member.member_id.starts_with("member-"));

            // Verify node_id matches
            prop_assert_eq!(&member.node_id, &registration.node_id);

            // Verify peer_id matches
            prop_assert_eq!(&member.peer_id, &registration.peer_id);

            // Verify capabilities match
            prop_assert_eq!(member.capabilities.gpu_memory_gb, registration.capabilities.gpu_memory_gb);
            prop_assert_eq!(member.capabilities.gpu_count, registration.capabilities.gpu_count);
            prop_assert_eq!(member.capabilities.cpu_cores, registration.capabilities.cpu_cores);
            prop_assert_eq!(member.capabilities.ram_gb, registration.capabilities.ram_gb);
            prop_assert_eq!(member.capabilities.max_concurrent, registration.capabilities.max_concurrent);
            prop_assert_eq!(member.capabilities.max_context_length, registration.capabilities.max_context_length);

            // Verify models match
            prop_assert_eq!(member.models.len(), registration.models.len());
            for (m1, m2) in member.models.iter().zip(registration.models.iter()) {
                prop_assert_eq!(&m1.model_id, &m2.model_id);
            }

            // Verify lightning pubkey matches
            prop_assert_eq!(&member.lightning_pubkey, &registration.lightning_pubkey);

            // Query the registry and verify
            let retrieved = registry.get_member(&registration.node_id);
            prop_assert!(retrieved.is_some(), "Member should be retrievable");

            let retrieved = retrieved.unwrap();
            prop_assert_eq!(&retrieved.node_id, &member.node_id);
            prop_assert_eq!(&retrieved.member_id, &member.member_id);
            prop_assert_eq!(retrieved.capabilities.gpu_memory_gb, member.capabilities.gpu_memory_gb);
            prop_assert_eq!(retrieved.models.len(), member.models.len());

            Ok(())
        })?;
    }

    /// Additional property: Duplicate registration fails
    #[test]
    fn duplicate_registration_fails(
        registration in registration_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = InMemoryPoolRegistry::with_registration_keys(vec!["valid-key".to_string()]);

            // First registration succeeds
            let result1 = registry.register_member(registration.clone()).await;
            prop_assert!(result1.is_ok());

            // Second registration fails
            let result2 = registry.register_member(registration).await;
            prop_assert!(result2.is_err());

            Ok(())
        })?;
    }

    /// Additional property: Invalid key fails registration
    #[test]
    fn invalid_key_fails_registration(
        registration in registration_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Registry with no valid keys
            let registry = InMemoryPoolRegistry::new();

            let result = registry.register_member(registration).await;
            prop_assert!(result.is_err());

            let err = result.unwrap_err();
            prop_assert_eq!(err.error_type, inference_pool::PoolErrorType::RegistrationError);

            Ok(())
        })?;
    }

    /// Additional property: Model query returns correct members
    #[test]
    fn model_query_returns_correct_members(
        registrations in prop::collection::vec(registration_strategy(), 2..5),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = InMemoryPoolRegistry::with_registration_keys(vec!["valid-key".to_string()]);

            // Register all members with unique node_ids
            let mut registered_nodes = Vec::new();
            for (i, mut reg) in registrations.into_iter().enumerate() {
                reg.node_id = format!("node-{}-{}", i, &reg.node_id[..8.min(reg.node_id.len())]);
                if let Ok(member) = registry.register_member(reg.clone()).await {
                    // Activate the member
                    registry.update_member_status(&member.node_id, MemberStatus::Active).await.ok();
                    registered_nodes.push((reg, member));
                }
            }

            // For each registered member, verify model queries work
            for (reg, _member) in &registered_nodes {
                for model in &reg.models {
                    let members_for_model = registry.get_members_for_model(&model.model_id);
                    // Should contain at least this member (might have others if same model)
                    prop_assert!(
                        members_for_model.iter().any(|m| m.node_id == reg.node_id),
                        "Member should be found in model query"
                    );
                }
            }

            Ok(())
        })?;
    }

    /// Additional property: Status updates are reflected correctly
    #[test]
    fn status_updates_are_reflected(
        registration in registration_strategy(),
        new_status in prop::sample::select(vec![
            MemberStatus::Active,
            MemberStatus::Inactive,
            MemberStatus::Draining,
            MemberStatus::AtRisk,
        ]),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = InMemoryPoolRegistry::with_registration_keys(vec!["valid-key".to_string()]);

            let member = registry.register_member(registration).await?;
            
            // Initial status is Pending
            let retrieved = registry.get_member(&member.node_id).unwrap();
            prop_assert_eq!(retrieved.status, MemberStatus::Pending);

            // Update status
            registry.update_member_status(&member.node_id, new_status).await?;

            // Verify status changed
            let retrieved = registry.get_member(&member.node_id).unwrap();
            prop_assert_eq!(retrieved.status, new_status);

            Ok(())
        })?;
    }

    /// Additional property: Metrics updates are reflected correctly
    #[test]
    fn metrics_updates_are_reflected(
        registration in registration_strategy(),
        avg_latency_ms in 0.0f64..10000.0f64,
        success_rate in 0.0f64..1.0f64,
        current_load in 0.0f64..1.0f64,
        jobs_completed in 0u64..1000000u64,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = InMemoryPoolRegistry::with_registration_keys(vec!["valid-key".to_string()]);

            let member = registry.register_member(registration).await?;
            
            let new_metrics = MemberMetrics {
                avg_latency_ms,
                success_rate,
                current_load,
                jobs_completed,
                jobs_failed: 0,
                tokens_processed: jobs_completed * 1000,
                consensus_participation: 0.95,
                uptime_percent: 99.9,
            };

            registry.update_member_metrics(&member.node_id, new_metrics.clone()).await?;

            let retrieved = registry.get_member(&member.node_id).unwrap();
            
            // Allow small floating point differences
            prop_assert!((retrieved.metrics.avg_latency_ms - avg_latency_ms).abs() < 0.001);
            prop_assert!((retrieved.metrics.success_rate - success_rate).abs() < 0.001);
            prop_assert!((retrieved.metrics.current_load - current_load).abs() < 0.001);
            prop_assert_eq!(retrieved.metrics.jobs_completed, jobs_completed);

            Ok(())
        })?;
    }

    /// Additional property: Remove member cleans up indices
    #[test]
    fn remove_member_cleans_up(
        registration in registration_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = InMemoryPoolRegistry::with_registration_keys(vec!["valid-key".to_string()]);

            let member = registry.register_member(registration.clone()).await?;
            
            // Verify member exists
            prop_assert!(registry.is_registered(&member.node_id));
            prop_assert!(registry.get_member(&member.node_id).is_some());

            // Remove member
            registry.remove_member(&member.node_id).await?;

            // Verify member is gone
            prop_assert!(!registry.is_registered(&member.node_id));
            prop_assert!(registry.get_member(&member.node_id).is_none());

            // Verify model index is cleaned up
            for model in &registration.models {
                let members_for_model = registry.get_members_for_model(&model.model_id);
                prop_assert!(
                    !members_for_model.iter().any(|m| m.node_id == member.node_id),
                    "Removed member should not appear in model queries"
                );
            }

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_count() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["key".to_string()]);

        assert_eq!(registry.total_count(), 0);
        assert_eq!(registry.count_by_status(MemberStatus::Active), 0);

        let reg = MemberRegistration {
            node_id: "node-1".to_string(),
            peer_id: "peer-1".to_string(),
            capabilities: NodeCapabilities::default(),
            models: vec![],
            lightning_pubkey: "pubkey".to_string(),
            multiaddrs: vec![],
            registration_key: "key".to_string(),
        };

        registry.register_member(reg).await.unwrap();
        assert_eq!(registry.total_count(), 1);
        assert_eq!(registry.count_by_status(MemberStatus::Pending), 1);

        registry.update_member_status("node-1", MemberStatus::Active).await.unwrap();
        assert_eq!(registry.count_by_status(MemberStatus::Active), 1);
        assert_eq!(registry.count_by_status(MemberStatus::Pending), 0);
    }

    #[tokio::test]
    async fn test_get_active_members_only() {
        let registry = InMemoryPoolRegistry::with_registration_keys(vec!["key".to_string()]);

        for i in 0..3 {
            let reg = MemberRegistration {
                node_id: format!("node-{}", i),
                peer_id: format!("peer-{}", i),
                capabilities: NodeCapabilities::default(),
                models: vec![],
                lightning_pubkey: "pubkey".to_string(),
                multiaddrs: vec![],
                registration_key: "key".to_string(),
            };
            registry.register_member(reg).await.unwrap();
        }

        // All pending initially
        assert_eq!(registry.get_active_members().len(), 0);

        // Activate one
        registry.update_member_status("node-0", MemberStatus::Active).await.unwrap();
        assert_eq!(registry.get_active_members().len(), 1);

        // Activate another
        registry.update_member_status("node-1", MemberStatus::Active).await.unwrap();
        assert_eq!(registry.get_active_members().len(), 2);

        // Mark one inactive
        registry.update_member_status("node-0", MemberStatus::Inactive).await.unwrap();
        assert_eq!(registry.get_active_members().len(), 1);
    }
}







