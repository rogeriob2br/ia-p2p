//! Property tests for Heartbeat Monitor.
//!
//! Feature: api-integration
//! Tests Property 2 from the design document.

use chrono::{Duration, Utc};
use inference_pool::heartbeat::{HeartbeatConfig, HeartbeatMonitor, HeartbeatMonitorTrait};
use inference_pool::models::{MemberMetrics, MemberStatus, ModelInfo, NodeCapabilities, PoolMember};
use inference_pool::registry::{InMemoryPoolRegistry, MemberRegistration, PoolRegistry};
use proptest::prelude::*;
use std::sync::Arc;

/// Strategy for generating heartbeat configurations
fn heartbeat_config_strategy() -> impl Strategy<Value = HeartbeatConfig> {
    (
        100u64..1000u64,    // interval_ms
        50u64..500u64,      // timeout_ms (will be adjusted based on interval)
        1u32..5u32,         // max_missed
        50u64..500u64,      // check_interval_ms
    )
        .prop_map(|(interval_ms, timeout_offset, max_missed, check_interval_ms)| {
            HeartbeatConfig {
                interval_ms,
                timeout_ms: interval_ms + timeout_offset, // timeout > interval
                max_missed,
                check_interval_ms,
            }
        })
}

/// Strategy for generating random node IDs
fn node_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("node-[a-z0-9]{8}")
        .unwrap()
        .prop_filter("node_id not empty", |s| !s.is_empty())
}

/// Create a test pool member
fn create_test_member(node_id: &str) -> PoolMember {
    PoolMember {
        member_id: format!("member-{}", node_id),
        node_id: node_id.to_string(),
        peer_id: format!("peer-{}", node_id),
        capabilities: NodeCapabilities::default(),
        models: vec![],
        status: MemberStatus::Active,
        last_heartbeat: Utc::now(),
        metrics: MemberMetrics::default(),
        lightning_pubkey: "pubkey".to_string(),
        multiaddrs: vec![],
        registered_at: Utc::now(),
        channel_id: None,
    }
}

/// Create a test registry with a member
async fn setup_registry_with_member(
    node_id: &str,
) -> (Arc<InMemoryPoolRegistry>, PoolMember) {
    let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
        "key".to_string(),
    ]));

    let reg = MemberRegistration {
        node_id: node_id.to_string(),
        peer_id: format!("peer-{}", node_id),
        capabilities: NodeCapabilities::default(),
        models: vec![],
        lightning_pubkey: "pubkey".to_string(),
        multiaddrs: vec![],
        registration_key: "key".to_string(),
    };

    let member = registry.register_member(reg).await.unwrap();
    registry
        .update_member_status(&member.node_id, MemberStatus::Active)
        .await
        .unwrap();

    (registry, member)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: api-integration
    /// Property 2: Heartbeat Timeout Marks Inactive
    ///
    /// For any Pool_Member that misses heartbeats for the configured timeout
    /// period, the Pool_Registry SHALL mark that member as inactive.
    ///
    /// Validates: Requirements 1.3, 1.4
    #[test]
    fn property_2_heartbeat_timeout_marks_inactive(
        node_id in node_id_strategy(),
        max_missed in 2u32..5u32,  // Use 2+ to have clear healthy -> unhealthy transition
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (registry, member) = setup_registry_with_member(&node_id).await;

            // Create monitor with short timeout for testing
            let config = HeartbeatConfig {
                interval_ms: 100,
                timeout_ms: 10, // Very short for testing
                max_missed,
                check_interval_ms: 100,
            };
            let monitor = HeartbeatMonitor::new(config, registry.clone());

            // Start monitoring
            monitor.start_monitoring(&member).await?;

            // Verify member starts healthy
            prop_assert!(monitor.is_healthy(&member.node_id));

            // Verify registry shows active
            let reg_member = registry.get_member(&member.node_id).unwrap();
            prop_assert_eq!(reg_member.status, MemberStatus::Active);

            // Simulate timeout by running check cycles without heartbeats
            // We need max_missed cycles to become unhealthy
            for i in 0..max_missed {
                // Wait for timeout
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

                // Run check cycle
                monitor.check_cycle().await;

                // Before reaching max_missed, should still be healthy
                if i + 1 < max_missed {
                    prop_assert!(
                        monitor.is_healthy(&member.node_id),
                        "Should be healthy at missed count {} (max={})",
                        i + 1,
                        max_missed
                    );
                }
            }

            // After max_missed cycles, should be unhealthy
            prop_assert!(
                !monitor.is_healthy(&member.node_id),
                "Should be unhealthy after {} missed heartbeats",
                max_missed
            );

            // Registry should show inactive
            let reg_member = registry.get_member(&member.node_id).unwrap();
            prop_assert_eq!(
                reg_member.status,
                MemberStatus::Inactive,
                "Registry should mark member as inactive"
            );

            // Unhealthy members list should contain this node
            let unhealthy = monitor.get_unhealthy_members();
            prop_assert!(
                unhealthy.contains(&member.node_id),
                "Member should be in unhealthy list"
            );

            Ok(())
        })?;
    }

    /// Additional property: Heartbeat resets missed count
    #[test]
    fn heartbeat_resets_missed_count(
        node_id in node_id_strategy(),
        missed_before_hb in 1u32..3u32,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (registry, member) = setup_registry_with_member(&node_id).await;

            let config = HeartbeatConfig {
                interval_ms: 100,
                timeout_ms: 10,
                max_missed: 5, // High enough that we don't become unhealthy
                check_interval_ms: 100,
            };
            let monitor = HeartbeatMonitor::new(config, registry.clone());

            monitor.start_monitoring(&member).await?;

            // Accumulate some missed heartbeats
            for _ in 0..missed_before_hb {
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                monitor.check_cycle().await;
            }

            // Verify missed count increased
            prop_assert!(
                monitor.get_missed_count(&member.node_id) >= missed_before_hb,
                "Missed count should be at least {}",
                missed_before_hb
            );

            // Send heartbeat
            monitor.process_heartbeat(&member.node_id, Utc::now()).await?;

            // Missed count should be reset
            prop_assert_eq!(
                monitor.get_missed_count(&member.node_id),
                0,
                "Missed count should be reset after heartbeat"
            );

            Ok(())
        })?;
    }

    /// Additional property: Recovery after heartbeat
    #[test]
    fn recovery_after_heartbeat(
        node_id in node_id_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (registry, member) = setup_registry_with_member(&node_id).await;

            let config = HeartbeatConfig {
                interval_ms: 100,
                timeout_ms: 10,
                max_missed: 1, // Become unhealthy quickly
                check_interval_ms: 100,
            };
            let monitor = HeartbeatMonitor::new(config, registry.clone());

            monitor.start_monitoring(&member).await?;

            // Make unhealthy
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            monitor.check_cycle().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            monitor.check_cycle().await;

            prop_assert!(!monitor.is_healthy(&member.node_id));

            let reg_member = registry.get_member(&member.node_id).unwrap();
            prop_assert_eq!(reg_member.status, MemberStatus::Inactive);

            // Send heartbeat to recover
            monitor.process_heartbeat(&member.node_id, Utc::now()).await?;

            // Should be healthy again
            prop_assert!(monitor.is_healthy(&member.node_id));
            prop_assert_eq!(monitor.get_missed_count(&member.node_id), 0);

            // Registry should show active
            let reg_member = registry.get_member(&member.node_id).unwrap();
            prop_assert_eq!(reg_member.status, MemberStatus::Active);

            // Should not be in unhealthy list
            let unhealthy = monitor.get_unhealthy_members();
            prop_assert!(!unhealthy.contains(&member.node_id));

            Ok(())
        })?;
    }

    /// Additional property: Multiple members tracked independently
    #[test]
    fn multiple_members_tracked_independently(
        node_count in 2usize..5usize,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
                "key".to_string(),
            ]));

            let config = HeartbeatConfig {
                interval_ms: 100,
                timeout_ms: 10,
                max_missed: 2,
                check_interval_ms: 100,
            };
            let monitor = HeartbeatMonitor::new(config, registry.clone());

            // Create and register multiple members
            let mut members = Vec::new();
            for i in 0..node_count {
                let reg = MemberRegistration {
                    node_id: format!("node-{}", i),
                    peer_id: format!("peer-{}", i),
                    capabilities: NodeCapabilities::default(),
                    models: vec![],
                    lightning_pubkey: "pubkey".to_string(),
                    multiaddrs: vec![],
                    registration_key: "key".to_string(),
                };
                let member = registry.register_member(reg).await.unwrap();
                registry
                    .update_member_status(&member.node_id, MemberStatus::Active)
                    .await
                    .unwrap();
                monitor.start_monitoring(&member).await.unwrap();
                members.push(member);
            }

            // Run timeout cycles
            for cycle in 0..3 {
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                
                // Send heartbeat for first member to keep it alive
                monitor
                    .process_heartbeat(&members[0].node_id, Utc::now())
                    .await
                    .unwrap();
                    
                monitor.check_cycle().await;
            }

            // First member should be healthy (received heartbeats)
            prop_assert!(
                monitor.is_healthy(&members[0].node_id),
                "First member should be healthy"
            );

            // Others should be unhealthy (missed heartbeats)
            for (i, member) in members.iter().skip(1).enumerate() {
                prop_assert!(
                    !monitor.is_healthy(&member.node_id),
                    "Member {} should be unhealthy",
                    i + 1
                );
            }

            Ok(())
        })?;
    }

    /// Additional property: Last heartbeat timestamp is accurate
    #[test]
    fn last_heartbeat_timestamp_is_accurate(
        node_id in node_id_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (registry, member) = setup_registry_with_member(&node_id).await;

            let config = HeartbeatConfig::default();
            let monitor = HeartbeatMonitor::new(config, registry.clone());

            monitor.start_monitoring(&member).await?;

            // Record timestamps
            let ts1 = Utc::now();
            monitor.process_heartbeat(&member.node_id, ts1).await?;
            let stored1 = monitor.get_last_heartbeat(&member.node_id).unwrap();
            prop_assert_eq!(stored1, ts1);

            // Small delay
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            let ts2 = Utc::now();
            monitor.process_heartbeat(&member.node_id, ts2).await?;
            let stored2 = monitor.get_last_heartbeat(&member.node_id).unwrap();
            prop_assert_eq!(stored2, ts2);

            // Second timestamp should be later
            prop_assert!(ts2 > ts1);
            prop_assert!(stored2 > stored1);

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_start_stop_monitoring() {
        let (registry, member) = setup_registry_with_member("test-node").await;
        let config = HeartbeatConfig::default();
        let monitor = HeartbeatMonitor::new(config, registry);

        // Start monitoring
        monitor.start_monitoring(&member).await.unwrap();
        assert!(monitor.is_healthy(&member.node_id));

        // Stop monitoring
        monitor.stop_monitoring(&member.node_id).await.unwrap();
        assert!(!monitor.is_healthy(&member.node_id)); // Not tracked = not healthy
    }

    #[tokio::test]
    async fn test_process_heartbeat_for_unknown_node() {
        let registry = Arc::new(InMemoryPoolRegistry::new());
        let config = HeartbeatConfig::default();
        let monitor = HeartbeatMonitor::new(config, registry);

        let result = monitor.process_heartbeat("unknown-node", Utc::now()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_monitor_start_stop() {
        let registry = Arc::new(InMemoryPoolRegistry::new());
        let config = HeartbeatConfig::default();
        let monitor = HeartbeatMonitor::new(config, registry);

        monitor.start().await.unwrap();
        monitor.stop().await.unwrap();
    }
}

