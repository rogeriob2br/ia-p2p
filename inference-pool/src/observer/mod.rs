//! Network Observer module.
//!
//! Receives orchestration events from the P2P network via libp2p gossipsub.
//! The Pool passively monitors the network - it does NOT control orchestration.
//!
//! ## Events Observed
//!
//! - Context group formation/dissolution
//! - Coordinator elections
//! - Consensus rounds
//! - Network topology changes
//! - Node join/leave events

mod observer;

pub use observer::{NetworkObserver, NetworkObserverConfig, ObserverEvent};

use crate::models::NetworkMap;
use async_trait::async_trait;

/// Observer trait for receiving network events
#[async_trait]
pub trait OrchestrationObserver: Send + Sync {
    /// Start observing the network
    async fn start(&self) -> crate::error::PoolResult<()>;

    /// Stop observing
    async fn stop(&self) -> crate::error::PoolResult<()>;

    /// Get current network map
    fn get_network_map(&self) -> NetworkMap;

    /// Subscribe to events
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ObserverEvent>;

    /// Check if observer is running
    fn is_running(&self) -> bool;
}
