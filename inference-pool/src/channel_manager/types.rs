//! Lightning Channel Manager types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Channel State
// ============================================================================

/// State of a Lightning channel
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelState {
    /// Channel opening in progress
    Opening,
    /// Channel pending confirmation
    PendingOpen,
    /// Channel is open and active
    Active,
    /// Channel inactive (peer offline)
    Inactive,
    /// Channel closing in progress
    Closing,
    /// Channel pending closure confirmation
    PendingClose,
    /// Channel closed
    Closed,
    /// Channel force-closed
    ForceClosed,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self::Opening
    }
}

impl ChannelState {
    /// Check if channel can process payments
    pub fn can_pay(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if channel is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Closed | Self::ForceClosed)
    }

    /// Check if channel is usable (active or has pending capacity)
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Active | Self::Inactive)
    }
}

// ============================================================================
// Channel
// ============================================================================

/// A Lightning channel with a pool member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Channel ID (funding txid:output_index)
    pub channel_id: String,

    /// Short channel ID (if confirmed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_channel_id: Option<String>,

    /// Remote peer public key
    pub peer_pubkey: String,

    /// Associated member ID in the pool
    pub member_id: String,

    /// Channel state
    pub state: ChannelState,

    /// Total channel capacity in satoshis
    pub capacity_sats: u64,

    /// Local balance in satoshis
    pub local_balance_sats: u64,

    /// Remote balance in satoshis
    pub remote_balance_sats: u64,

    /// Channel reserve (minimum local balance)
    pub reserve_sats: u64,

    /// Number of confirmations
    pub confirmations: u32,

    /// Whether we initiated the channel
    pub initiator: bool,

    /// Whether channel is private
    pub private: bool,

    /// Funding transaction ID
    pub funding_txid: String,

    /// Funding output index
    pub funding_output: u32,

    /// Channel opened timestamp
    pub opened_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,

    /// Total payments sent through channel
    pub total_sent_sats: u64,

    /// Total payments received through channel
    pub total_received_sats: u64,

    /// Number of HTLCs sent
    pub htlcs_sent: u64,

    /// Number of HTLCs received
    pub htlcs_received: u64,
}

impl Channel {
    /// Get available outbound capacity
    pub fn outbound_capacity(&self) -> u64 {
        self.local_balance_sats.saturating_sub(self.reserve_sats)
    }

    /// Get available inbound capacity
    pub fn inbound_capacity(&self) -> u64 {
        self.remote_balance_sats.saturating_sub(self.reserve_sats)
    }

    /// Check if can send a specific amount
    pub fn can_send(&self, amount_sats: u64) -> bool {
        self.state.can_pay() && self.outbound_capacity() >= amount_sats
    }

    /// Check if can receive a specific amount
    pub fn can_receive(&self, amount_sats: u64) -> bool {
        self.state.can_pay() && self.inbound_capacity() >= amount_sats
    }

    /// Get channel utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        if self.capacity_sats == 0 {
            return 0.0;
        }
        self.local_balance_sats as f64 / self.capacity_sats as f64
    }
}

// ============================================================================
// Channel Configuration
// ============================================================================

/// Configuration for channel opening
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Default channel capacity in satoshis
    pub default_capacity_sats: u64,

    /// Minimum channel capacity
    pub min_capacity_sats: u64,

    /// Maximum channel capacity
    pub max_capacity_sats: u64,

    /// Channel reserve percentage (0-100)
    pub reserve_percent: f64,

    /// Target number of confirmations
    pub target_confirmations: u32,

    /// Whether to open private channels
    pub private_channels: bool,

    /// Push amount on channel open (satoshis)
    pub push_amount_sats: u64,

    /// Fee rate for channel open (sat/vbyte)
    pub fee_rate_sat_vbyte: u64,

    /// Maximum pending HTLCs
    pub max_pending_htlcs: u32,

    /// Minimum HTLC value
    pub min_htlc_msat: u64,

    /// Maximum HTLC value
    pub max_htlc_msat: u64,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            default_capacity_sats: 1_000_000,     // 0.01 BTC
            min_capacity_sats: 100_000,           // 0.001 BTC
            max_capacity_sats: 100_000_000,       // 1 BTC
            reserve_percent: 1.0,                 // 1% reserve
            target_confirmations: 3,
            private_channels: true,
            push_amount_sats: 0,
            fee_rate_sat_vbyte: 10,
            max_pending_htlcs: 30,
            min_htlc_msat: 1000,                  // 1 sat minimum
            max_htlc_msat: 500_000_000,           // 0.5 BTC max
        }
    }
}

// ============================================================================
// Channel Stats
// ============================================================================

/// Aggregated channel statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelStats {
    /// Total number of channels
    pub total_channels: u32,

    /// Number of active channels
    pub active_channels: u32,

    /// Number of pending channels
    pub pending_channels: u32,

    /// Number of inactive channels
    pub inactive_channels: u32,

    /// Total capacity across all channels
    pub total_capacity_sats: u64,

    /// Total local balance
    pub total_local_balance_sats: u64,

    /// Total remote balance
    pub total_remote_balance_sats: u64,

    /// Total outbound capacity available
    pub total_outbound_capacity_sats: u64,

    /// Total inbound capacity available
    pub total_inbound_capacity_sats: u64,

    /// Total sent through all channels
    pub total_sent_sats: u64,

    /// Total received through all channels
    pub total_received_sats: u64,
}

// ============================================================================
// Payment Types
// ============================================================================

/// Payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Unique payment ID
    pub payment_id: String,

    /// Destination public key
    pub destination_pubkey: String,

    /// Amount in satoshis
    pub amount_sats: u64,

    /// Payment hash (if paying an invoice)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_hash: Option<String>,

    /// Description/memo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timeout in seconds
    pub timeout_secs: u32,

    /// Maximum fee in satoshis
    pub max_fee_sats: u64,

    /// Preferred channel ID (for direct payments)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    /// Associated member ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<String>,
}

/// Payment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    /// Payment ID
    pub payment_id: String,

    /// Payment hash
    pub payment_hash: String,

    /// Payment preimage (if successful)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_preimage: Option<String>,

    /// Amount sent in satoshis
    pub amount_sats: u64,

    /// Fee paid in satoshis
    pub fee_sats: u64,

    /// Payment state
    pub state: PaymentState,

    /// Number of hops in the route
    pub hops: u32,

    /// Whether payment was made via direct channel
    pub direct_channel: bool,

    /// Channel ID used (if direct)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    /// Error message if failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Payment timestamp
    pub timestamp: DateTime<Utc>,
}

/// Payment state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentState {
    /// Payment pending
    Pending,
    /// Payment in flight
    InFlight,
    /// Payment succeeded
    Succeeded,
    /// Payment failed
    Failed,
    /// Payment timed out
    TimedOut,
}

impl Default for PaymentState {
    fn default() -> Self {
        Self::Pending
    }
}

// ============================================================================
// Channel Events
// ============================================================================

/// Channel events for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelEvent {
    /// Channel opened
    Opened {
        channel_id: String,
        peer_pubkey: String,
        capacity_sats: u64,
    },
    /// Channel became active
    Active {
        channel_id: String,
    },
    /// Channel became inactive
    Inactive {
        channel_id: String,
        reason: String,
    },
    /// Channel closing
    Closing {
        channel_id: String,
        cooperative: bool,
    },
    /// Channel closed
    Closed {
        channel_id: String,
        reason: String,
    },
    /// Payment sent
    PaymentSent {
        channel_id: String,
        amount_sats: u64,
        payment_hash: String,
    },
    /// Payment received
    PaymentReceived {
        channel_id: String,
        amount_sats: u64,
        payment_hash: String,
    },
    /// Liquidity low warning
    LiquidityLow {
        channel_id: String,
        available_sats: u64,
        threshold_sats: u64,
    },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_state_can_pay() {
        assert!(ChannelState::Active.can_pay());
        assert!(!ChannelState::Opening.can_pay());
        assert!(!ChannelState::Inactive.can_pay());
        assert!(!ChannelState::Closed.can_pay());
    }

    #[test]
    fn test_channel_state_is_terminal() {
        assert!(ChannelState::Closed.is_terminal());
        assert!(ChannelState::ForceClosed.is_terminal());
        assert!(!ChannelState::Active.is_terminal());
    }

    #[test]
    fn test_channel_capacity_calculations() {
        let channel = Channel {
            channel_id: "abc:0".to_string(),
            short_channel_id: None,
            peer_pubkey: "02abc...".to_string(),
            member_id: "member-1".to_string(),
            state: ChannelState::Active,
            capacity_sats: 1_000_000,
            local_balance_sats: 600_000,
            remote_balance_sats: 400_000,
            reserve_sats: 10_000,
            confirmations: 6,
            initiator: true,
            private: true,
            funding_txid: "abc123".to_string(),
            funding_output: 0,
            opened_at: Utc::now(),
            last_activity: Utc::now(),
            total_sent_sats: 0,
            total_received_sats: 0,
            htlcs_sent: 0,
            htlcs_received: 0,
        };

        assert_eq!(channel.outbound_capacity(), 590_000);
        assert_eq!(channel.inbound_capacity(), 390_000);
        assert!(channel.can_send(500_000));
        assert!(!channel.can_send(600_000));
        assert!(channel.can_receive(300_000));
        assert!(!channel.can_receive(400_000));
        assert!((channel.utilization() - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_payment_state_serialization() {
        assert_eq!(
            serde_json::to_string(&PaymentState::Succeeded).unwrap(),
            "\"succeeded\""
        );
        assert_eq!(
            serde_json::to_string(&PaymentState::Failed).unwrap(),
            "\"failed\""
        );
    }

    #[test]
    fn test_channel_event_serialization() {
        let event = ChannelEvent::Opened {
            channel_id: "abc:0".to_string(),
            peer_pubkey: "02abc".to_string(),
            capacity_sats: 1_000_000,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"opened\""));
        assert!(json.contains("\"capacity_sats\":1000000"));
    }

    #[test]
    fn test_default_channel_config() {
        let config = ChannelConfig::default();
        assert_eq!(config.default_capacity_sats, 1_000_000);
        assert!(config.private_channels);
    }
}






