//! Heartbeat Monitor module.
//!
//! Monitors availability of Pool_Members via periodic heartbeats.
//!
//! ## Responsibilities
//!
//! - Track heartbeat timestamps for each member
//! - Detect members that miss heartbeats
//! - Mark members as inactive after timeout
//! - Emit events when member status changes

mod monitor;

pub use monitor::{HeartbeatConfig, HeartbeatMonitor, HeartbeatMonitorImpl};

use crate::error::PoolResult;
use crate::models::PoolMember;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Heartbeat event
#[derive(Debug, Clone)]
pub enum HeartbeatEvent {
    /// Heartbeat received from a member
    Received {
        node_id: String,
        timestamp: DateTime<Utc>,
        latency_ms: u64,
    },
    /// Member missed heartbeat
    Missed {
        node_id: String,
        missed_count: u32,
        last_heartbeat: DateTime<Utc>,
    },
    /// Member became unhealthy (exceeded max missed heartbeats)
    Unhealthy {
        node_id: String,
        missed_count: u32,
        last_heartbeat: DateTime<Utc>,
    },
    /// Member recovered (received heartbeat after being unhealthy)
    Recovered {
        node_id: String,
        timestamp: DateTime<Utc>,
    },
}

/// Heartbeat Monitor trait
#[async_trait]
pub trait HeartbeatMonitorTrait: Send + Sync {
    /// Start monitoring a member
    async fn start_monitoring(&self, member: &PoolMember) -> PoolResult<()>;

    /// Stop monitoring a member
    async fn stop_monitoring(&self, node_id: &str) -> PoolResult<()>;

    /// Process a heartbeat response
    async fn process_heartbeat(&self, node_id: &str, timestamp: DateTime<Utc>) -> PoolResult<()>;

    /// Get members that missed heartbeats
    fn get_unhealthy_members(&self) -> Vec<String>;

    /// Get the number of missed heartbeats for a member
    fn get_missed_count(&self, node_id: &str) -> u32;

    /// Get the last heartbeat timestamp for a member
    fn get_last_heartbeat(&self, node_id: &str) -> Option<DateTime<Utc>>;

    /// Check if a member is healthy
    fn is_healthy(&self, node_id: &str) -> bool;

    /// Run a check cycle (detect missed heartbeats)
    async fn check_cycle(&self) -> Vec<HeartbeatEvent>;

    /// Start the background monitoring loop
    async fn start(&self) -> PoolResult<()>;

    /// Stop the background monitoring loop
    async fn stop(&self) -> PoolResult<()>;
}







