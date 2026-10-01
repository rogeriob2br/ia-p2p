//! Heartbeat monitor implementation.

use super::{HeartbeatEvent, HeartbeatMonitorTrait};
use crate::error::{PoolError, PoolResult};
use crate::models::PoolMember;
use crate::registry::PoolRegistry;
use crate::models::MemberStatus;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Heartbeat configuration
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Expected interval between heartbeats (ms)
    pub interval_ms: u64,

    /// Timeout before considering heartbeat missed (ms)
    pub timeout_ms: u64,

    /// Maximum number of missed heartbeats before marking unhealthy
    pub max_missed: u32,

    /// Check interval for the monitor (ms)
    pub check_interval_ms: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 30_000,      // 30 seconds
            timeout_ms: 45_000,       // 45 seconds (1.5x interval)
            max_missed: 3,            // 3 missed = unhealthy
            check_interval_ms: 15_000, // Check every 15 seconds
        }
    }
}

/// State for a monitored member
#[derive(Debug, Clone)]
struct MemberState {
    /// Last heartbeat received
    last_heartbeat: DateTime<Utc>,

    /// Number of consecutive missed heartbeats
    missed_count: u32,

    /// Whether the member is currently healthy
    is_healthy: bool,

    /// When monitoring started
    started_at: DateTime<Utc>,
}

impl MemberState {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            last_heartbeat: now,
            missed_count: 0,
            is_healthy: true,
            started_at: now,
        }
    }
}

/// Heartbeat Monitor implementation
pub struct HeartbeatMonitor<R: PoolRegistry> {
    /// Configuration
    config: HeartbeatConfig,

    /// Member states indexed by node_id
    states: DashMap<String, MemberState>,

    /// Registry reference for updating member status
    registry: Arc<R>,

    /// Event broadcaster
    event_tx: broadcast::Sender<HeartbeatEvent>,

    /// Running flag
    running: AtomicBool,
}

impl<R: PoolRegistry + 'static> HeartbeatMonitor<R> {
    /// Create a new heartbeat monitor
    pub fn new(config: HeartbeatConfig, registry: Arc<R>) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            config,
            states: DashMap::new(),
            registry,
            event_tx,
            running: AtomicBool::new(false),
        }
    }

    /// Get the event receiver
    pub fn subscribe(&self) -> broadcast::Receiver<HeartbeatEvent> {
        self.event_tx.subscribe()
    }

    /// Emit an event
    fn emit(&self, event: HeartbeatEvent) {
        // Ignore send errors (no receivers)
        let _ = self.event_tx.send(event);
    }

    /// Check if heartbeat is overdue for a member
    fn is_overdue(&self, state: &MemberState) -> bool {
        let timeout = Duration::milliseconds(self.config.timeout_ms as i64);
        Utc::now() - state.last_heartbeat > timeout
    }
}

#[async_trait]
impl<R: PoolRegistry + 'static> HeartbeatMonitorTrait for HeartbeatMonitor<R> {
    async fn start_monitoring(&self, member: &PoolMember) -> PoolResult<()> {
        let node_id = &member.node_id;

        if self.states.contains_key(node_id) {
            tracing::debug!(node_id = %node_id, "Already monitoring member");
            return Ok(());
        }

        self.states.insert(node_id.clone(), MemberState::new());

        tracing::info!(
            node_id = %node_id,
            interval_ms = self.config.interval_ms,
            "Started monitoring member heartbeat"
        );

        Ok(())
    }

    async fn stop_monitoring(&self, node_id: &str) -> PoolResult<()> {
        self.states.remove(node_id);

        tracing::info!(node_id = %node_id, "Stopped monitoring member heartbeat");

        Ok(())
    }

    async fn process_heartbeat(&self, node_id: &str, timestamp: DateTime<Utc>) -> PoolResult<()> {
        let mut state = self
            .states
            .get_mut(node_id)
            .ok_or_else(|| PoolError::node_not_found(node_id))?;

        let was_unhealthy = !state.is_healthy;
        let latency_ms = (Utc::now() - timestamp).num_milliseconds().max(0) as u64;

        // Update state
        state.last_heartbeat = timestamp;
        state.missed_count = 0;
        state.is_healthy = true;

        // Emit received event
        self.emit(HeartbeatEvent::Received {
            node_id: node_id.to_string(),
            timestamp,
            latency_ms,
        });

        // If was unhealthy, emit recovered event
        if was_unhealthy {
            self.emit(HeartbeatEvent::Recovered {
                node_id: node_id.to_string(),
                timestamp,
            });

            // Update registry status
            self.registry
                .update_member_status(node_id, MemberStatus::Active)
                .await?;

            tracing::info!(node_id = %node_id, "Member recovered from unhealthy state");
        }

        // Update registry heartbeat timestamp
        self.registry.update_heartbeat(node_id, timestamp).await?;

        tracing::trace!(
            node_id = %node_id,
            latency_ms = latency_ms,
            "Processed heartbeat"
        );

        Ok(())
    }

    fn get_unhealthy_members(&self) -> Vec<String> {
        self.states
            .iter()
            .filter(|entry| !entry.value().is_healthy)
            .map(|entry| entry.key().clone())
            .collect()
    }

    fn get_missed_count(&self, node_id: &str) -> u32 {
        self.states
            .get(node_id)
            .map(|s| s.missed_count)
            .unwrap_or(0)
    }

    fn get_last_heartbeat(&self, node_id: &str) -> Option<DateTime<Utc>> {
        self.states.get(node_id).map(|s| s.last_heartbeat)
    }

    fn is_healthy(&self, node_id: &str) -> bool {
        self.states
            .get(node_id)
            .map(|s| s.is_healthy)
            .unwrap_or(false)
    }

    async fn check_cycle(&self) -> Vec<HeartbeatEvent> {
        let mut events = Vec::new();

        for mut entry in self.states.iter_mut() {
            let node_id = entry.key().clone();
            let state = entry.value_mut();

            if self.is_overdue(state) {
                state.missed_count += 1;

                if state.missed_count >= self.config.max_missed && state.is_healthy {
                    // Became unhealthy
                    state.is_healthy = false;

                    let event = HeartbeatEvent::Unhealthy {
                        node_id: node_id.clone(),
                        missed_count: state.missed_count,
                        last_heartbeat: state.last_heartbeat,
                    };
                    events.push(event.clone());
                    self.emit(event);

                    // Update registry status
                    if let Err(e) = self
                        .registry
                        .update_member_status(&node_id, MemberStatus::Inactive)
                        .await
                    {
                        tracing::error!(
                            node_id = %node_id,
                            error = %e,
                            "Failed to update member status to inactive"
                        );
                    }

                    tracing::warn!(
                        node_id = %node_id,
                        missed_count = state.missed_count,
                        last_heartbeat = %state.last_heartbeat,
                        "Member became unhealthy"
                    );
                } else if state.is_healthy {
                    // Still healthy but missed
                    let event = HeartbeatEvent::Missed {
                        node_id: node_id.clone(),
                        missed_count: state.missed_count,
                        last_heartbeat: state.last_heartbeat,
                    };
                    events.push(event.clone());
                    self.emit(event);

                    tracing::debug!(
                        node_id = %node_id,
                        missed_count = state.missed_count,
                        "Member missed heartbeat"
                    );
                }
            }
        }

        events
    }

    async fn start(&self) -> PoolResult<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        tracing::info!(
            check_interval_ms = self.config.check_interval_ms,
            "Heartbeat monitor started"
        );

        Ok(())
    }

    async fn stop(&self) -> PoolResult<()> {
        self.running.store(false, Ordering::SeqCst);

        tracing::info!("Heartbeat monitor stopped");

        Ok(())
    }
}

/// Type alias for the default implementation
pub type HeartbeatMonitorImpl = HeartbeatMonitor<crate::registry::InMemoryPoolRegistry>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MemberMetrics, ModelInfo, NodeCapabilities};
    use crate::registry::{InMemoryPoolRegistry, MemberRegistration};

    fn create_test_member() -> PoolMember {
        PoolMember {
            member_id: "member-1".to_string(),
            node_id: "node-1".to_string(),
            peer_id: "peer-1".to_string(),
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

    async fn setup_registry_with_member() -> (Arc<InMemoryPoolRegistry>, PoolMember) {
        let registry = Arc::new(InMemoryPoolRegistry::with_registration_keys(vec![
            "key".to_string(),
        ]));

        let reg = MemberRegistration {
            node_id: "node-1".to_string(),
            peer_id: "peer-1".to_string(),
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

    #[tokio::test]
    async fn test_start_stop_monitoring() {
        let (registry, member) = setup_registry_with_member().await;
        let config = HeartbeatConfig::default();
        let monitor = HeartbeatMonitor::new(config, registry);

        monitor.start_monitoring(&member).await.unwrap();
        assert!(monitor.is_healthy(&member.node_id));
        assert_eq!(monitor.get_missed_count(&member.node_id), 0);

        monitor.stop_monitoring(&member.node_id).await.unwrap();
        assert!(!monitor.is_healthy(&member.node_id));
    }

    #[tokio::test]
    async fn test_process_heartbeat() {
        let (registry, member) = setup_registry_with_member().await;
        let config = HeartbeatConfig::default();
        let monitor = HeartbeatMonitor::new(config, registry);

        monitor.start_monitoring(&member).await.unwrap();

        let now = Utc::now();
        monitor.process_heartbeat(&member.node_id, now).await.unwrap();

        let last_hb = monitor.get_last_heartbeat(&member.node_id).unwrap();
        assert_eq!(last_hb, now);
    }

    #[tokio::test]
    async fn test_check_cycle_detects_missed() {
        let (registry, member) = setup_registry_with_member().await;
        let config = HeartbeatConfig {
            interval_ms: 100,
            timeout_ms: 50, // Very short timeout for testing
            max_missed: 3,
            check_interval_ms: 100,
        };
        let monitor = HeartbeatMonitor::new(config, registry);

        monitor.start_monitoring(&member).await.unwrap();

        // Wait for timeout
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let events = monitor.check_cycle().await;
        assert!(!events.is_empty());

        // Check missed count increased
        assert!(monitor.get_missed_count(&member.node_id) >= 1);
    }

    #[tokio::test]
    async fn test_becomes_unhealthy_after_max_missed() {
        let (registry, member) = setup_registry_with_member().await;
        let config = HeartbeatConfig {
            interval_ms: 100,
            timeout_ms: 10,
            max_missed: 2,
            check_interval_ms: 100,
        };
        let monitor = HeartbeatMonitor::new(config, registry.clone());

        monitor.start_monitoring(&member).await.unwrap();

        // Run check cycles until unhealthy
        for _ in 0..3 {
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            monitor.check_cycle().await;
        }

        // Should be unhealthy now
        assert!(!monitor.is_healthy(&member.node_id));
        assert!(monitor.get_unhealthy_members().contains(&member.node_id));

        // Registry should show inactive
        let reg_member = registry.get_member(&member.node_id).unwrap();
        assert_eq!(reg_member.status, MemberStatus::Inactive);
    }

    #[tokio::test]
    async fn test_recovery_after_heartbeat() {
        let (registry, member) = setup_registry_with_member().await;
        let config = HeartbeatConfig {
            interval_ms: 100,
            timeout_ms: 10,
            max_missed: 1,
            check_interval_ms: 100,
        };
        let monitor = HeartbeatMonitor::new(config, registry.clone());

        monitor.start_monitoring(&member).await.unwrap();

        // Make unhealthy
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        monitor.check_cycle().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        monitor.check_cycle().await;

        assert!(!monitor.is_healthy(&member.node_id));

        // Send heartbeat to recover
        monitor
            .process_heartbeat(&member.node_id, Utc::now())
            .await
            .unwrap();

        assert!(monitor.is_healthy(&member.node_id));
        assert_eq!(monitor.get_missed_count(&member.node_id), 0);

        // Registry should show active
        let reg_member = registry.get_member(&member.node_id).unwrap();
        assert_eq!(reg_member.status, MemberStatus::Active);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let (registry, member) = setup_registry_with_member().await;
        let config = HeartbeatConfig::default();
        let monitor = Arc::new(HeartbeatMonitor::new(config, registry));

        let mut receiver = monitor.subscribe();

        monitor.start_monitoring(&member).await.unwrap();
        monitor
            .process_heartbeat(&member.node_id, Utc::now())
            .await
            .unwrap();

        // Should receive the heartbeat event
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            receiver.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        match event {
            HeartbeatEvent::Received { node_id, .. } => {
                assert_eq!(node_id, member.node_id);
            }
            _ => panic!("Expected Received event"),
        }
    }
}







