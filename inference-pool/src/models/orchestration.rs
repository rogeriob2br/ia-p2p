//! Orchestration event types.
//!
//! These types represent events from the distributed orchestration
//! that the Pool monitors via P2P.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ============================================================================
// Orchestration Events
// ============================================================================

/// Events from the distributed orchestration that Pool monitors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum OrchestrationEvent {
    /// Emergent coordinator changed
    CoordinatorChanged {
        old_coordinator: Option<String>,
        new_coordinator: String,
        affinity_score: f64,
        reason: CoordinatorChangeReason,
        timestamp: DateTime<Utc>,
    },

    /// Context group was formed
    ContextGroupFormed {
        group_id: String,
        context: String,
        leader: String,
        members: Vec<String>,
        specialization: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// Context group was dissolved
    ContextGroupDissolved {
        group_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Group leader changed
    GroupLeaderChanged {
        group_id: String,
        old_leader: Option<String>,
        new_leader: String,
        timestamp: DateTime<Utc>,
    },

    /// Consensus was reached
    ConsensusReached {
        decision_id: String,
        decision_type: String,
        participants: Vec<String>,
        rounds: u32,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },

    /// Node role changed
    NodeRoleChanged {
        node_id: String,
        old_role: NodeRole,
        new_role: NodeRole,
        timestamp: DateTime<Utc>,
    },

    /// Network partition detected
    NetworkPartitionDetected {
        partition_id: String,
        affected_nodes: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// Network partition resolved
    NetworkPartitionResolved {
        partition_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Node metrics update
    NodeMetricsUpdate {
        node_id: String,
        metrics: NodeMetrics,
        timestamp: DateTime<Utc>,
    },
}

impl OrchestrationEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::CoordinatorChanged { timestamp, .. }
            | Self::ContextGroupFormed { timestamp, .. }
            | Self::ContextGroupDissolved { timestamp, .. }
            | Self::GroupLeaderChanged { timestamp, .. }
            | Self::ConsensusReached { timestamp, .. }
            | Self::NodeRoleChanged { timestamp, .. }
            | Self::NetworkPartitionDetected { timestamp, .. }
            | Self::NetworkPartitionResolved { timestamp, .. }
            | Self::NodeMetricsUpdate { timestamp, .. } => *timestamp,
        }
    }

    /// Get event type as string
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::CoordinatorChanged { .. } => "coordinator_changed",
            Self::ContextGroupFormed { .. } => "context_group_formed",
            Self::ContextGroupDissolved { .. } => "context_group_dissolved",
            Self::GroupLeaderChanged { .. } => "group_leader_changed",
            Self::ConsensusReached { .. } => "consensus_reached",
            Self::NodeRoleChanged { .. } => "node_role_changed",
            Self::NetworkPartitionDetected { .. } => "network_partition_detected",
            Self::NetworkPartitionResolved { .. } => "network_partition_resolved",
            Self::NodeMetricsUpdate { .. } => "node_metrics_update",
        }
    }
}

/// Reason for coordinator change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoordinatorChangeReason {
    /// Affinity score changed
    AffinityChange,
    /// Previous coordinator failed
    NodeFailure,
    /// Periodic rotation
    PeriodicRotation,
    /// Load balancing
    LoadBalancing,
    /// Manual change
    Manual,
}

/// Node role in the distributed orchestration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeRole {
    /// Regular node
    Regular,
    /// Emergent coordinator
    EmergentCoordinator,
    /// Group leader
    GroupLeader { group_id: String },
    /// Multiple roles
    MultiRole { roles: Vec<String> },
}

impl Default for NodeRole {
    fn default() -> Self {
        Self::Regular
    }
}

/// Node metrics from distributed orchestration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetrics {
    /// Success rate
    pub success_rate: f64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
    /// Current load (0.0 - 1.0)
    pub current_load: f64,
    /// Jobs completed
    pub jobs_completed: u64,
    /// Jobs failed
    pub jobs_failed: u64,
    /// Tokens processed
    pub tokens_processed: u64,
    /// Consensus participation rate
    pub consensus_participation: f64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

// ============================================================================
// Network Map
// ============================================================================

/// Network topology map that Pool maintains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMap {
    /// Current emergent coordinator
    pub current_coordinator: Option<String>,

    /// Coordinator affinity score
    pub coordinator_affinity: f64,

    /// Active context groups
    pub context_groups: HashMap<String, ContextGroup>,

    /// Node connections
    pub connections: Vec<NodeConnection>,

    /// Aggregated network metrics
    pub metrics: NetworkMetrics,

    /// Event history (last N events)
    pub event_history: VecDeque<OrchestrationEvent>,

    /// Maximum event history size
    #[serde(skip)]
    pub max_history_size: usize,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for NetworkMap {
    fn default() -> Self {
        Self {
            current_coordinator: None,
            coordinator_affinity: 0.0,
            context_groups: HashMap::new(),
            connections: Vec::new(),
            metrics: NetworkMetrics::default(),
            event_history: VecDeque::new(),
            max_history_size: 1000,
            last_updated: Utc::now(),
        }
    }
}

impl NetworkMap {
    /// Create a new network map with specified history size
    pub fn with_history_size(max_history_size: usize) -> Self {
        Self {
            max_history_size,
            ..Default::default()
        }
    }

    /// Add an event to history
    pub fn add_event(&mut self, event: OrchestrationEvent) {
        self.event_history.push_front(event);
        while self.event_history.len() > self.max_history_size {
            self.event_history.pop_back();
        }
        self.last_updated = Utc::now();
    }

    /// Get recent events
    pub fn get_recent_events(&self, limit: usize) -> Vec<&OrchestrationEvent> {
        self.event_history.iter().take(limit).collect()
    }

    /// Process an orchestration event and update the map
    pub fn process_event(&mut self, event: OrchestrationEvent) {
        match &event {
            OrchestrationEvent::CoordinatorChanged {
                new_coordinator,
                affinity_score,
                ..
            } => {
                self.current_coordinator = Some(new_coordinator.clone());
                self.coordinator_affinity = *affinity_score;
            }

            OrchestrationEvent::ContextGroupFormed {
                group_id,
                context,
                leader,
                members,
                specialization,
                timestamp,
            } => {
                self.context_groups.insert(
                    group_id.clone(),
                    ContextGroup {
                        group_id: group_id.clone(),
                        context: context.clone(),
                        leader: leader.clone(),
                        members: members.clone(),
                        specialization: specialization.clone(),
                        formed_at: *timestamp,
                        jobs_processed: 0,
                        avg_latency_ms: 0.0,
                        status: GroupStatus::Active,
                    },
                );
                self.metrics.active_groups = self.context_groups.len() as u32;
            }

            OrchestrationEvent::ContextGroupDissolved { group_id, .. } => {
                if let Some(group) = self.context_groups.get_mut(group_id) {
                    group.status = GroupStatus::Inactive;
                }
                self.context_groups.remove(group_id);
                self.metrics.active_groups = self.context_groups.len() as u32;
            }

            OrchestrationEvent::GroupLeaderChanged {
                group_id,
                new_leader,
                ..
            } => {
                if let Some(group) = self.context_groups.get_mut(group_id) {
                    group.leader = new_leader.clone();
                }
            }

            OrchestrationEvent::ConsensusReached { rounds, .. } => {
                // Update consensus metrics
                let total = self.metrics.consensus_rounds_total + 1;
                self.metrics.consensus_rounds_avg = (self.metrics.consensus_rounds_avg
                    * (total - 1) as f64
                    + *rounds as f64)
                    / total as f64;
                self.metrics.consensus_rounds_total = total;
                self.metrics.consensus_success_rate =
                    (self.metrics.consensus_success_rate * (total - 1) as f64 + 1.0) / total as f64;
            }

            OrchestrationEvent::NetworkPartitionDetected { .. } => {
                self.metrics.partition_events += 1;
            }

            _ => {}
        }

        self.add_event(event);
    }
}

/// Active context group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGroup {
    /// Group identifier
    pub group_id: String,

    /// Context/domain
    pub context: String,

    /// Current leader
    pub leader: String,

    /// Group members
    pub members: Vec<String>,

    /// Specializations
    pub specialization: Vec<String>,

    /// When the group was formed
    pub formed_at: DateTime<Utc>,

    /// Jobs processed by this group
    pub jobs_processed: u64,

    /// Average latency for this group
    pub avg_latency_ms: f64,

    /// Group status
    pub status: GroupStatus,
}

/// Group status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupStatus {
    /// Active and processing jobs
    Active,
    /// Being formed
    Forming,
    /// Being dissolved
    Dissolving,
    /// Inactive
    Inactive,
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    /// Source node
    pub from_node: String,

    /// Destination node
    pub to_node: String,

    /// Latency between nodes
    pub latency_ms: f64,

    /// Messages exchanged
    pub messages_exchanged: u64,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// Aggregated network metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    /// Total nodes in network
    pub total_nodes: u32,

    /// Active coordinators
    pub active_coordinators: u32,

    /// Active context groups
    pub active_groups: u32,

    /// Average consensus rounds
    pub consensus_rounds_avg: f64,

    /// Total consensus rounds
    #[serde(skip)]
    pub consensus_rounds_total: u64,

    /// Average decision latency
    pub decision_latency_avg_ms: f64,

    /// Number of partition events
    pub partition_events: u32,

    /// Consensus success rate
    pub consensus_success_rate: f64,
}

// ============================================================================
// Topology Changes
// ============================================================================

/// Topology change event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum TopologyChange {
    /// Node was added
    NodeAdded { node_id: String },
    /// Node was removed
    NodeRemoved { node_id: String },
    /// Connection was established
    ConnectionEstablished { from: String, to: String },
    /// Connection was lost
    ConnectionLost { from: String, to: String },
}

// ============================================================================
// API Responses
// ============================================================================

/// Network map response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMapResponse {
    /// Current coordinator
    pub current_coordinator: Option<String>,

    /// Coordinator affinity
    pub coordinator_affinity: f64,

    /// Total nodes
    pub total_nodes: u32,

    /// Active groups
    pub active_groups: u32,

    /// Context groups
    pub context_groups: Vec<ContextGroupInfo>,

    /// Network health
    pub network_health: NetworkHealthInfo,

    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Context group info for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGroupInfo {
    /// Group ID
    pub group_id: String,

    /// Context
    pub context: String,

    /// Leader
    pub leader: String,

    /// Member count
    pub member_count: u32,

    /// Specializations
    pub specialization: Vec<String>,

    /// Jobs processed
    pub jobs_processed: u64,

    /// Average latency
    pub avg_latency_ms: f64,

    /// Status
    pub status: GroupStatus,
}

impl From<&ContextGroup> for ContextGroupInfo {
    fn from(group: &ContextGroup) -> Self {
        Self {
            group_id: group.group_id.clone(),
            context: group.context.clone(),
            leader: group.leader.clone(),
            member_count: group.members.len() as u32,
            specialization: group.specialization.clone(),
            jobs_processed: group.jobs_processed,
            avg_latency_ms: group.avg_latency_ms,
            status: group.status,
        }
    }
}

/// Network health info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthInfo {
    /// Consensus success rate
    pub consensus_success_rate: f64,

    /// Average consensus rounds
    pub avg_consensus_rounds: f64,

    /// Partition events in last 24h
    pub partition_events_last_24h: u32,

    /// Coordinator rotations in last 24h
    pub coordinator_rotations_last_24h: u32,
}

/// Orchestration events response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationEventsResponse {
    /// Events
    pub events: Vec<OrchestrationEvent>,

    /// Total count
    pub total_count: u64,

    /// Page number
    pub page: u32,

    /// Page size
    pub page_size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestration_event_serialization() {
        let event = OrchestrationEvent::CoordinatorChanged {
            old_coordinator: Some("node-1".to_string()),
            new_coordinator: "node-2".to_string(),
            affinity_score: 0.95,
            reason: CoordinatorChangeReason::AffinityChange,
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event_type\":\"coordinator_changed\""));
        assert!(json.contains("\"new_coordinator\":\"node-2\""));

        let parsed: OrchestrationEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type(), "coordinator_changed");
    }

    #[test]
    fn test_node_role_serialization() {
        let regular = NodeRole::Regular;
        assert_eq!(serde_json::to_string(&regular).unwrap(), "\"regular\"");

        let leader = NodeRole::GroupLeader {
            group_id: "group-1".to_string(),
        };
        let json = serde_json::to_string(&leader).unwrap();
        assert!(json.contains("\"group_leader\""));
        assert!(json.contains("\"group_id\":\"group-1\""));
    }

    #[test]
    fn test_network_map_process_event() {
        let mut map = NetworkMap::default();

        // Process coordinator change
        map.process_event(OrchestrationEvent::CoordinatorChanged {
            old_coordinator: None,
            new_coordinator: "node-1".to_string(),
            affinity_score: 0.9,
            reason: CoordinatorChangeReason::AffinityChange,
            timestamp: Utc::now(),
        });

        assert_eq!(map.current_coordinator, Some("node-1".to_string()));
        assert_eq!(map.coordinator_affinity, 0.9);
        assert_eq!(map.event_history.len(), 1);

        // Process group formation
        map.process_event(OrchestrationEvent::ContextGroupFormed {
            group_id: "group-1".to_string(),
            context: "nlp".to_string(),
            leader: "node-1".to_string(),
            members: vec!["node-1".to_string(), "node-2".to_string()],
            specialization: vec!["text".to_string()],
            timestamp: Utc::now(),
        });

        assert_eq!(map.context_groups.len(), 1);
        assert!(map.context_groups.contains_key("group-1"));
        assert_eq!(map.metrics.active_groups, 1);
    }

    #[test]
    fn test_network_map_event_history_limit() {
        let mut map = NetworkMap::with_history_size(3);

        for i in 0..5 {
            map.process_event(OrchestrationEvent::CoordinatorChanged {
                old_coordinator: None,
                new_coordinator: format!("node-{}", i),
                affinity_score: 0.9,
                reason: CoordinatorChangeReason::AffinityChange,
                timestamp: Utc::now(),
            });
        }

        assert_eq!(map.event_history.len(), 3);
    }

    #[test]
    fn test_group_status_serialization() {
        assert_eq!(
            serde_json::to_string(&GroupStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&GroupStatus::Forming).unwrap(),
            "\"forming\""
        );
    }
}







