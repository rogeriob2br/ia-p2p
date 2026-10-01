//! LND API types.
//!
//! Types matching LND's gRPC/REST API for channel and payment operations.

use serde::{Deserialize, Serialize};

// ============================================================================
// Node Info
// ============================================================================

/// LND GetInfo response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInfo {
    /// Node identity public key
    pub identity_pubkey: String,

    /// Node alias
    pub alias: String,

    /// Number of pending channels
    pub num_pending_channels: u32,

    /// Number of active channels
    pub num_active_channels: u32,

    /// Number of inactive channels
    pub num_inactive_channels: u32,

    /// Number of peers
    pub num_peers: u32,

    /// Block height
    pub block_height: u32,

    /// Block hash
    pub block_hash: String,

    /// Whether synced to chain
    pub synced_to_chain: bool,

    /// Whether synced to graph
    pub synced_to_graph: bool,

    /// LND version
    pub version: String,

    /// Network (mainnet, testnet, regtest)
    #[serde(default)]
    pub chains: Vec<LndChain>,
}

/// Chain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndChain {
    pub chain: String,
    pub network: String,
}

// ============================================================================
// Wallet Balance
// ============================================================================

/// Wallet balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    /// Total confirmed balance in satoshis
    pub total_balance: i64,

    /// Confirmed balance
    pub confirmed_balance: i64,

    /// Unconfirmed balance
    pub unconfirmed_balance: i64,

    /// Locked balance (in pending channels)
    #[serde(default)]
    pub locked_balance: i64,
}

/// Channel balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBalance {
    /// Total local balance across all channels
    pub local_balance: Option<LndAmount>,

    /// Total remote balance across all channels
    pub remote_balance: Option<LndAmount>,

    /// Unsettled local balance
    pub unsettled_local_balance: Option<LndAmount>,

    /// Unsettled remote balance
    pub unsettled_remote_balance: Option<LndAmount>,

    /// Pending open local balance
    pub pending_open_local_balance: Option<LndAmount>,

    /// Pending open remote balance
    pub pending_open_remote_balance: Option<LndAmount>,
}

/// Amount with sat and msat
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LndAmount {
    pub sat: i64,
    pub msat: i64,
}

// ============================================================================
// Channel Operations
// ============================================================================

/// Open channel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenChannelRequest {
    /// Node pubkey to connect to (hex encoded)
    pub node_pubkey_string: String,

    /// Local funding amount in satoshis
    pub local_funding_amount: i64,

    /// Push amount to remote in satoshis
    #[serde(default)]
    pub push_sat: i64,

    /// Target confirmations for funding tx
    #[serde(default)]
    pub target_conf: i32,

    /// Satoshis per vbyte for funding tx
    #[serde(default)]
    pub sat_per_vbyte: i64,

    /// Whether channel should be private
    #[serde(default)]
    pub private: bool,

    /// Minimum HTLC in millisatoshis
    #[serde(default)]
    pub min_htlc_msat: i64,

    /// Remote CSV delay
    #[serde(default)]
    pub remote_csv_delay: u32,

    /// Minimum confirmations for each UTXO
    #[serde(default)]
    pub min_confs: i32,

    /// Whether to spend unconfirmed outputs
    #[serde(default)]
    pub spend_unconfirmed: bool,
}

/// Channel point (funding tx outpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPoint {
    /// Funding transaction ID
    pub funding_txid_str: String,

    /// Output index
    pub output_index: u32,
}

impl ChannelPoint {
    /// Parse from "txid:index" format
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            Some(Self {
                funding_txid_str: parts[0].to_string(),
                output_index: parts[1].parse().ok()?,
            })
        } else {
            None
        }
    }

    /// Convert to "txid:index" format
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.funding_txid_str, self.output_index)
    }
}

/// LND Channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndChannel {
    /// Whether channel is active
    pub active: bool,

    /// Remote pubkey
    pub remote_pubkey: String,

    /// Channel point
    pub channel_point: String,

    /// Channel ID (short channel id)
    #[serde(default)]
    pub chan_id: String,

    /// Channel capacity in satoshis
    pub capacity: i64,

    /// Local balance in satoshis
    pub local_balance: i64,

    /// Remote balance in satoshis
    pub remote_balance: i64,

    /// Commit fee in satoshis
    #[serde(default)]
    pub commit_fee: i64,

    /// Commit weight
    #[serde(default)]
    pub commit_weight: i64,

    /// Fee per kw
    #[serde(default)]
    pub fee_per_kw: i64,

    /// Unsettled balance
    #[serde(default)]
    pub unsettled_balance: i64,

    /// Total satoshis sent
    #[serde(default)]
    pub total_satoshis_sent: i64,

    /// Total satoshis received
    #[serde(default)]
    pub total_satoshis_received: i64,

    /// Number of updates
    #[serde(default)]
    pub num_updates: u64,

    /// Pending HTLCs
    #[serde(default)]
    pub pending_htlcs: Vec<LndHtlc>,

    /// CSV delay
    #[serde(default)]
    pub csv_delay: u32,

    /// Whether channel is private
    #[serde(default)]
    pub private: bool,

    /// Whether we initiated
    #[serde(default)]
    pub initiator: bool,

    /// Local channel reserve
    #[serde(default)]
    pub local_chan_reserve_sat: i64,

    /// Remote channel reserve
    #[serde(default)]
    pub remote_chan_reserve_sat: i64,

    /// Number of confirmations
    #[serde(default)]
    pub confirmation_height: u32,

    /// Lifetime (seconds)
    #[serde(default)]
    pub lifetime: i64,

    /// Uptime (seconds)
    #[serde(default)]
    pub uptime: i64,
}

/// HTLC info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndHtlc {
    pub incoming: bool,
    pub amount: i64,
    pub hash_lock: String,
    pub expiration_height: u32,
    pub htlc_index: u64,
}

/// List channels response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListChannelsResponse {
    pub channels: Vec<LndChannel>,
}

/// Pending channels response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChannelsResponse {
    /// Total limbo balance
    pub total_limbo_balance: i64,

    /// Pending open channels
    #[serde(default)]
    pub pending_open_channels: Vec<PendingOpenChannel>,

    /// Pending closing channels
    #[serde(default)]
    pub pending_closing_channels: Vec<PendingCloseChannel>,

    /// Pending force closing channels
    #[serde(default)]
    pub pending_force_closing_channels: Vec<PendingForceCloseChannel>,

    /// Waiting close channels
    #[serde(default)]
    pub waiting_close_channels: Vec<WaitingCloseChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOpenChannel {
    pub channel: PendingChannelInfo,
    pub confirmation_height: u32,
    pub commit_fee: i64,
    pub commit_weight: i64,
    pub fee_per_kw: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCloseChannel {
    pub channel: PendingChannelInfo,
    pub closing_txid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingForceCloseChannel {
    pub channel: PendingChannelInfo,
    pub closing_txid: String,
    pub limbo_balance: i64,
    pub maturity_height: u32,
    pub blocks_til_maturity: i32,
    pub recovered_balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingCloseChannel {
    pub channel: PendingChannelInfo,
    pub limbo_balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChannelInfo {
    pub remote_node_pub: String,
    pub channel_point: String,
    pub capacity: i64,
    pub local_balance: i64,
    pub remote_balance: i64,
    #[serde(default)]
    pub local_chan_reserve_sat: i64,
    #[serde(default)]
    pub remote_chan_reserve_sat: i64,
    #[serde(default)]
    pub initiator: String,
    #[serde(default)]
    pub commitment_type: String,
}

/// Close channel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseChannelRequest {
    /// Channel point
    pub channel_point: ChannelPoint,

    /// Force close
    #[serde(default)]
    pub force: bool,

    /// Target confirmations
    #[serde(default)]
    pub target_conf: i32,

    /// Sat per vbyte
    #[serde(default)]
    pub sat_per_vbyte: i64,
}

// ============================================================================
// Payment Operations
// ============================================================================

/// Send payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPaymentRequest {
    /// Payment request (BOLT11 invoice)
    #[serde(default)]
    pub payment_request: String,

    /// Destination public key (for keysend)
    #[serde(default)]
    pub dest: String,

    /// Amount in satoshis (if not in invoice)
    #[serde(default)]
    pub amt: i64,

    /// Payment hash (hex)
    #[serde(default)]
    pub payment_hash: String,

    /// Final CLTV delta
    #[serde(default)]
    pub final_cltv_delta: i32,

    /// Fee limit in satoshis
    #[serde(default)]
    pub fee_limit_sat: i64,

    /// Outgoing channel ID (for specific channel)
    #[serde(default)]
    pub outgoing_chan_id: String,

    /// Timeout in seconds
    #[serde(default)]
    pub timeout_seconds: i32,

    /// Allow self payment
    #[serde(default)]
    pub allow_self_payment: bool,

    /// Max number of shards for MPP
    #[serde(default)]
    pub max_parts: u32,
}

/// Send payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPaymentResponse {
    /// Payment hash
    pub payment_hash: String,

    /// Payment preimage
    #[serde(default)]
    pub payment_preimage: String,

    /// Payment route
    #[serde(default)]
    pub payment_route: Option<PaymentRoute>,

    /// Payment error
    #[serde(default)]
    pub payment_error: String,
}

/// Payment route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRoute {
    /// Total time lock
    pub total_time_lock: u32,

    /// Total fees in millisatoshis
    pub total_fees_msat: i64,

    /// Total amount in millisatoshis
    pub total_amt_msat: i64,

    /// Route hops
    pub hops: Vec<RouteHop>,
}

/// Route hop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteHop {
    /// Channel ID
    pub chan_id: String,

    /// Pub key
    pub pub_key: String,

    /// Amount to forward in millisatoshis
    pub amt_to_forward_msat: i64,

    /// Fee in millisatoshis
    pub fee_msat: i64,

    /// Expiry
    pub expiry: u32,
}

/// Payment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndPayment {
    /// Payment hash
    pub payment_hash: String,

    /// Value in satoshis
    pub value_sat: i64,

    /// Creation date (unix timestamp)
    pub creation_date: i64,

    /// Fee in satoshis
    pub fee_sat: i64,

    /// Payment preimage
    pub payment_preimage: String,

    /// Payment request (invoice)
    #[serde(default)]
    pub payment_request: String,

    /// Status (UNKNOWN, IN_FLIGHT, SUCCEEDED, FAILED)
    pub status: String,

    /// Failure reason
    #[serde(default)]
    pub failure_reason: String,

    /// Payment index
    #[serde(default)]
    pub payment_index: u64,
}

/// List payments response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPaymentsResponse {
    pub payments: Vec<LndPayment>,
    #[serde(default)]
    pub first_index_offset: u64,
    #[serde(default)]
    pub last_index_offset: u64,
}

// ============================================================================
// Invoice Operations
// ============================================================================

/// Add invoice request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInvoiceRequest {
    /// Memo/description
    #[serde(default)]
    pub memo: String,

    /// Amount in satoshis
    pub value: i64,

    /// Expiry in seconds
    #[serde(default)]
    pub expiry: i64,

    /// Private routing hints
    #[serde(default)]
    pub private: bool,
}

/// Add invoice response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInvoiceResponse {
    /// Payment hash
    pub r_hash: String,

    /// Payment request (BOLT11)
    pub payment_request: String,

    /// Add index
    pub add_index: u64,

    /// Payment address
    #[serde(default)]
    pub payment_addr: String,
}

/// Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInvoice {
    /// Memo
    pub memo: String,

    /// Payment hash
    pub r_hash: String,

    /// Payment preimage
    #[serde(default)]
    pub r_preimage: String,

    /// Value in satoshis
    pub value: i64,

    /// Amount paid in satoshis
    #[serde(default)]
    pub amt_paid_sat: i64,

    /// Creation date (unix timestamp)
    pub creation_date: i64,

    /// Settle date
    #[serde(default)]
    pub settle_date: i64,

    /// Payment request
    #[serde(default)]
    pub payment_request: String,

    /// Expiry
    pub expiry: i64,

    /// Whether settled
    pub settled: bool,

    /// State (OPEN, SETTLED, CANCELED, ACCEPTED)
    #[serde(default)]
    pub state: String,
}

// ============================================================================
// Peer Operations
// ============================================================================

/// Connect peer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectPeerRequest {
    /// Peer address
    pub addr: LightningAddress,

    /// Whether permanent
    #[serde(default)]
    pub perm: bool,

    /// Timeout
    #[serde(default)]
    pub timeout: u64,
}

/// Lightning address (pubkey@host:port)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningAddress {
    /// Public key
    pub pubkey: String,

    /// Host (ip:port)
    pub host: String,
}

/// Peer info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndPeer {
    /// Public key
    pub pub_key: String,

    /// Address
    pub address: String,

    /// Bytes sent
    pub bytes_sent: u64,

    /// Bytes received
    pub bytes_recv: u64,

    /// Satoshis sent
    pub sat_sent: i64,

    /// Satoshis received
    pub sat_recv: i64,

    /// Inbound
    pub inbound: bool,

    /// Ping time in microseconds
    pub ping_time: i64,

    /// Sync type
    #[serde(default)]
    pub sync_type: String,
}

/// List peers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPeersResponse {
    pub peers: Vec<LndPeer>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_point_parsing() {
        let point = ChannelPoint::from_string("abc123:0").unwrap();
        assert_eq!(point.funding_txid_str, "abc123");
        assert_eq!(point.output_index, 0);
        assert_eq!(point.to_string(), "abc123:0");
    }

    #[test]
    fn test_lnd_info_deserialization() {
        let json = r#"{
            "identity_pubkey": "02abc",
            "alias": "my-node",
            "num_pending_channels": 1,
            "num_active_channels": 5,
            "num_inactive_channels": 0,
            "num_peers": 10,
            "block_height": 800000,
            "block_hash": "000000",
            "synced_to_chain": true,
            "synced_to_graph": true,
            "version": "0.17.0"
        }"#;

        let info: LndInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.identity_pubkey, "02abc");
        assert_eq!(info.num_active_channels, 5);
    }

    #[test]
    fn test_lnd_channel_deserialization() {
        let json = r#"{
            "active": true,
            "remote_pubkey": "02def",
            "channel_point": "abc:0",
            "capacity": 1000000,
            "local_balance": 500000,
            "remote_balance": 500000,
            "private": true,
            "initiator": true
        }"#;

        let channel: LndChannel = serde_json::from_str(json).unwrap();
        assert!(channel.active);
        assert_eq!(channel.capacity, 1000000);
    }
}






