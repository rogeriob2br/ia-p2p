//! Network Observer implementation.

use crate::error::PoolResult;
use crate::models::{ContextGroup, NetworkMap, NodeRole, OrchestrationEvent};
use crate::registry::PoolRegistry;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Observer event types
#[derive(Debug, Clone)]
pub enum ObserverEvent {
    /// Orchestration event received
    Orchestration(OrchestrationEvent),

    /// Network map updated
    MapUpdated(NetworkMap),

    /// Connection to P2P network established
    Connected,

    /// Connection to P2P network lost
    Disconnected,

    /// Error occurred
    Error(String),
}

/// Network Observer configuration
#[derive(Debug, Clone)]
pub struct NetworkObserverConfig {
    /// P2P listen addresses
    pub listen_addrs: Vec<String>,

    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,

    /// Orchestration events topic
    pub orchestration_topic: String,

    /// Heartbeat topic
    pub heartbeat_topic: String,

    /// Event broadcast channel capacity
    pub event_channel_capacity: usize,

    /// Maximum event history
    pub max_event_history: usize,
}

impl Default for NetworkObserverConfig {
    fn default() -> Self {
        Self {
            listen_addrs: vec!["/ip4/0.0.0.0/tcp/9000".to_string()],
            bootstrap_nodes: vec![],
            orchestration_topic: "/pool/orchestration/1.0.0".to_string(),
            heartbeat_topic: "/pool/heartbeat/1.0.0".to_string(),
            event_channel_capacity: 1000,
            max_event_history: 100,
        }
    }
}

/// Network Observer
///
/// Receives and processes orchestration events from the P2P network.
/// Updates the network map based on observed events.
pub struct NetworkObserver<R: PoolRegistry> {
    /// Configuration
    config: NetworkObserverConfig,

    /// Pool registry
    _registry: Arc<R>,

    /// Current network map
    network_map: Arc<RwLock<NetworkMap>>,

    /// Event broadcast channel
    event_tx: broadcast::Sender<ObserverEvent>,

    /// Running flag
    running: AtomicBool,

    /// Event history
    event_history: Arc<RwLock<Vec<OrchestrationEvent>>>,
}

impl<R: PoolRegistry> NetworkObserver<R> {
    /// Create a new network observer
    pub fn new(config: NetworkObserverConfig, registry: Arc<R>) -> Self {
        let (event_tx, _) = broadcast::channel(config.event_channel_capacity);

        Self {
            config,
            _registry: registry,
            network_map: Arc::new(RwLock::new(NetworkMap::default())),
            event_tx,
            running: AtomicBool::new(false),
            event_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Process an orchestration event
    pub async fn process_event(&self, event: OrchestrationEvent) -> PoolResult<()> {
        // Store in history
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());

            // Trim history if needed
            if history.len() > self.config.max_event_history {
                history.remove(0);
            }
        }

        // Update network map based on event
        {
            let mut map = self.network_map.write().await;
            map.process_event(event.clone());
        }

        // Broadcast event
        let _ = self.event_tx.send(ObserverEvent::Orchestration(event));

        // Broadcast updated map
        let map = self.network_map.read().await.clone();
        let _ = self.event_tx.send(ObserverEvent::MapUpdated(map));

        Ok(())
    }

    /// Get context groups from the network map
    pub async fn get_context_groups(&self) -> Vec<ContextGroup> {
        let map = self.network_map.read().await;
        map.context_groups.values().cloned().collect()
    }

    /// Get current coordinator
    pub async fn get_coordinator(&self) -> Option<String> {
        let map = self.network_map.read().await;
        map.current_coordinator.clone()
    }

    /// Get event history
    pub async fn get_event_history(&self) -> Vec<OrchestrationEvent> {
        self.event_history.read().await.clone()
    }

    /// Simulate receiving an event (for testing without actual P2P)
    pub async fn inject_event(&self, event: OrchestrationEvent) -> PoolResult<()> {
        self.process_event(event).await
    }
}

#[async_trait]
impl<R: PoolRegistry + 'static> super::OrchestrationObserver for NetworkObserver<R> {
    async fn start(&self) -> PoolResult<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        tracing::info!(
            topics = ?[&self.config.orchestration_topic, &self.config.heartbeat_topic],
            "Starting network observer"
        );

        self.running.store(true, Ordering::SeqCst);

        // Broadcast connected event
        let _ = self.event_tx.send(ObserverEvent::Connected);

        // TODO: Initialize libp2p and subscribe to gossipsub topics
        // For now, we just mark as running

        Ok(())
    }

    async fn stop(&self) -> PoolResult<()> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        tracing::info!("Stopping network observer");

        self.running.store(false, Ordering::SeqCst);

        // Broadcast disconnected event
        let _ = self.event_tx.send(ObserverEvent::Disconnected);

        Ok(())
    }

    fn get_network_map(&self) -> NetworkMap {
        // Use blocking read for sync interface
        futures::executor::block_on(async { self.network_map.read().await.clone() })
    }

    fn subscribe(&self) -> broadcast::Receiver<ObserverEvent> {
        self.event_tx.subscribe()
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CoordinatorChangeReason;
    use crate::observer::OrchestrationObserver;
    use crate::registry::InMemoryPoolRegistry;
    use chrono::Utc;

    fn create_test_observer() -> NetworkObserver<InMemoryPoolRegistry> {
        let registry = Arc::new(InMemoryPoolRegistry::new());
        NetworkObserver::new(NetworkObserverConfig::default(), registry)
    }

    #[tokio::test]
    async fn test_start_stop() {
        let observer = create_test_observer();

        assert!(!observer.is_running());

        observer.start().await.unwrap();
        assert!(observer.is_running());

        observer.stop().await.unwrap();
        assert!(!observer.is_running());
    }

    #[tokio::test]
    async fn test_process_event() {
        let observer = create_test_observer();

        let event = OrchestrationEvent::ContextGroupFormed {
            group_id: "group-1".to_string(),
            context: "test-context".to_string(),
            leader: "node-1".to_string(),
            members: vec!["node-1".to_string(), "node-2".to_string()],
            specialization: vec!["general".to_string()],
            timestamp: Utc::now(),
        };

        observer.process_event(event.clone()).await.unwrap();

        let history = observer.get_event_history().await;
        assert_eq!(history.len(), 1);

        let groups = observer.get_context_groups().await;
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].group_id, "group-1");
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let observer = create_test_observer();
        let mut rx = observer.subscribe();

        let event = OrchestrationEvent::CoordinatorChanged {
            old_coordinator: Some("node-1".to_string()),
            new_coordinator: "node-2".to_string(),
            affinity_score: 0.95,
            reason: CoordinatorChangeReason::AffinityChange,
            timestamp: Utc::now(),
        };

        observer.process_event(event).await.unwrap();

        // Should receive orchestration event
        let received = rx.recv().await.unwrap();
        assert!(matches!(received, ObserverEvent::Orchestration(_)));

        // Should receive map updated event
        let received = rx.recv().await.unwrap();
        assert!(matches!(received, ObserverEvent::MapUpdated(_)));
    }

    #[tokio::test]
    async fn test_event_history_limit() {
        let config = NetworkObserverConfig {
            max_event_history: 5,
            ..Default::default()
        };
        let registry = Arc::new(InMemoryPoolRegistry::new());
        let observer = NetworkObserver::new(config, registry);

        // Add more events than the limit
        for i in 0..10 {
            let event = OrchestrationEvent::ContextGroupFormed {
                group_id: format!("group-{}", i),
                context: format!("context-{}", i),
                leader: "node-1".to_string(),
                members: vec!["node-1".to_string()],
                specialization: vec!["general".to_string()],
                timestamp: Utc::now(),
            };
            observer.process_event(event).await.unwrap();
        }

        let history = observer.get_event_history().await;
        assert_eq!(history.len(), 5); // Limited to max
    }

    #[tokio::test]
    async fn test_coordinator_tracking() {
        let observer = create_test_observer();

        // Initially no coordinator
        assert!(observer.get_coordinator().await.is_none());

        // Inject coordinator change event
        let event = OrchestrationEvent::CoordinatorChanged {
            old_coordinator: None,
            new_coordinator: "node-1".to_string(),
            affinity_score: 0.9,
            reason: CoordinatorChangeReason::AffinityChange,
            timestamp: Utc::now(),
        };
        observer.inject_event(event).await.unwrap();

        // Now should have coordinator
        let coordinator = observer.get_coordinator().await;
        assert_eq!(coordinator, Some("node-1".to_string()));
    }
}
