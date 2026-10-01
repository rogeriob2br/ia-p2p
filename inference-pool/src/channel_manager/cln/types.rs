//! CLN (Core Lightning) API types.
//!
//! Types matching CLN's JSON-RPC API for channel and payment operations.

use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC
// ============================================================================

/// JSON-RPC request
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

impl<T> JsonRpcRequest<T> {
    pub fn new(id: u64, method: &str, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        }
    }
}

/// JSON-RPC response
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// Node Info
// ============================================================================

/// CLN getinfo response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnInfo {
    /// Node ID (public key)
    pub id: String,

    /// Node alias
    pub alias: String,

    /// Color
    pub color: String,

    /// Number of peers
    pub num_peers: u32,

    /// Number of pending channels
    pub num_pending_channels: u32,

    /// Number of active channels
    pub num_active_channels: u32,

    /// Number of inactive channels
    pub num_inactive_channels: u32,

    /// Network (bitcoin, testnet, regtest)
    pub network: String,

    /// Block height
    pub blockheight: u32,

    /// Warning if any
    #[serde(default)]
    pub warning_bitcoind_sync: Option<String>,

    /// Warning if any
    #[serde(default)]
    pub warning_lightningd_sync: Option<String>,

    /// Addresses
    #[serde(default)]
    pub address: Vec<ClnAddress>,

    /// Binding addresses
    #[serde(default)]
    pub binding: Vec<ClnBinding>,

    /// Version
    pub version: String,
}

/// CLN address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnAddress {
    #[serde(rename = "type")]
    pub addr_type: String,
    pub address: String,
    pub port: u16,
}

/// CLN binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnBinding {
    #[serde(rename = "type")]
    pub bind_type: String,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub socket: Option<String>,
}

// ============================================================================
// Funds
// ============================================================================

/// List funds response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFundsResponse {
    pub outputs: Vec<ClnOutput>,
    pub channels: Vec<ClnChannelFunds>,
}

/// On-chain output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnOutput {
    pub txid: String,
    pub output: u32,
    pub amount_msat: ClnAmount,
    pub scriptpubkey: String,
    #[serde(default)]
    pub address: Option<String>,
    pub status: String,
    #[serde(default)]
    pub reserved: bool,
    #[serde(default)]
    pub blockheight: Option<u32>,
}

/// Channel funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnChannelFunds {
    pub peer_id: String,
    pub our_amount_msat: ClnAmount,
    pub amount_msat: ClnAmount,
    pub funding_txid: String,
    pub funding_output: u32,
    pub connected: bool,
    pub state: String,
    #[serde(default)]
    pub short_channel_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
}

/// CLN amount (can be string "123msat" or number)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClnAmount {
    Msat(u64),
    String(String),
}

impl ClnAmount {
    /// Get value in millisatoshis
    pub fn msat(&self) -> u64 {
        match self {
            ClnAmount::Msat(v) => *v,
            ClnAmount::String(s) => {
                // Parse "123msat" format
                s.trim_end_matches("msat")
                    .parse()
                    .unwrap_or(0)
            }
        }
    }

    /// Get value in satoshis
    pub fn sat(&self) -> u64 {
        self.msat() / 1000
    }
}

// ============================================================================
// Channels
// ============================================================================

/// List peer channels response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPeerChannelsResponse {
    pub channels: Vec<ClnChannel>,
}

/// CLN Channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnChannel {
    /// Peer ID
    pub peer_id: String,

    /// Whether peer is connected
    pub peer_connected: bool,

    /// Channel state
    pub state: String,

    /// Short channel ID
    #[serde(default)]
    pub short_channel_id: Option<String>,

    /// Channel ID
    #[serde(default)]
    pub channel_id: Option<String>,

    /// Funding transaction ID
    pub funding_txid: String,

    /// Funding output index
    pub funding_outnum: u32,

    /// Whether we opened this channel
    #[serde(default)]
    pub opener: Option<String>,

    /// Whether channel is private
    #[serde(default)]
    pub private: bool,

    /// Total channel capacity
    pub total_msat: ClnAmount,

    /// Our balance
    pub to_us_msat: ClnAmount,

    /// Their balance
    #[serde(default)]
    pub to_them_msat: Option<ClnAmount>,

    /// Our reserve
    #[serde(default)]
    pub our_reserve_msat: Option<ClnAmount>,

    /// Their reserve
    #[serde(default)]
    pub their_reserve_msat: Option<ClnAmount>,

    /// Spendable by us
    #[serde(default)]
    pub spendable_msat: Option<ClnAmount>,

    /// Receivable by us
    #[serde(default)]
    pub receivable_msat: Option<ClnAmount>,

    /// Minimum HTLC
    #[serde(default)]
    pub minimum_htlc_in_msat: Option<ClnAmount>,

    /// Maximum HTLC
    #[serde(default)]
    pub maximum_htlc_out_msat: Option<ClnAmount>,

    /// Number of HTLCs
    #[serde(default)]
    pub htlcs: Vec<ClnHtlc>,

    /// Fee base
    #[serde(default)]
    pub fee_base_msat: Option<ClnAmount>,

    /// Fee proportional
    #[serde(default)]
    pub fee_proportional_millionths: Option<u32>,

    /// Dust limit
    #[serde(default)]
    pub dust_limit_msat: Option<ClnAmount>,

    /// Max accepted HTLCs
    #[serde(default)]
    pub max_accepted_htlcs: Option<u32>,

    /// In payments offered
    #[serde(default)]
    pub in_payments_offered: Option<u64>,

    /// In offered amount
    #[serde(default)]
    pub in_offered_msat: Option<ClnAmount>,

    /// In payments fulfilled
    #[serde(default)]
    pub in_payments_fulfilled: Option<u64>,

    /// In fulfilled amount
    #[serde(default)]
    pub in_fulfilled_msat: Option<ClnAmount>,

    /// Out payments offered
    #[serde(default)]
    pub out_payments_offered: Option<u64>,

    /// Out offered amount
    #[serde(default)]
    pub out_offered_msat: Option<ClnAmount>,

    /// Out payments fulfilled
    #[serde(default)]
    pub out_payments_fulfilled: Option<u64>,

    /// Out fulfilled amount
    #[serde(default)]
    pub out_fulfilled_msat: Option<ClnAmount>,
}

/// CLN HTLC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnHtlc {
    pub direction: String,
    pub id: u64,
    pub amount_msat: ClnAmount,
    pub expiry: u32,
    pub payment_hash: String,
    pub state: String,
}

/// Open channel request params
#[derive(Debug, Clone, Serialize)]
pub struct FundChannelParams {
    /// Peer ID
    pub id: String,

    /// Amount (can be "all" or satoshi amount)
    pub amount: String,

    /// Fee rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feerate: Option<String>,

    /// Announce (public channel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announce: Option<bool>,

    /// Minimum confirmations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minconf: Option<u32>,

    /// Push amount in millisatoshis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_msat: Option<u64>,

    /// Channel reserve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve: Option<String>,
}

/// Fund channel response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundChannelResponse {
    pub tx: String,
    pub txid: String,
    pub outnum: u32,
    pub channel_id: String,
    #[serde(default)]
    pub close_to: Option<String>,
    #[serde(default)]
    pub mindepth: Option<u32>,
}

/// Close channel params
#[derive(Debug, Clone, Serialize)]
pub struct CloseParams {
    /// Channel ID or peer ID
    pub id: String,

    /// Timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unilateraltimeout: Option<u32>,

    /// Destination address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,

    /// Fee negotiation start
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_negotiation_step: Option<String>,

    /// Force lease expiration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_lease_closed: Option<bool>,
}

/// Close channel response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseResponse {
    #[serde(rename = "type")]
    pub close_type: String,
    #[serde(default)]
    pub tx: Option<String>,
    #[serde(default)]
    pub txid: Option<String>,
}

// ============================================================================
// Payments
// ============================================================================

/// Pay invoice params
#[derive(Debug, Clone, Serialize)]
pub struct PayParams {
    /// BOLT11 invoice
    pub bolt11: String,

    /// Amount in millisatoshis (if not in invoice)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_msat: Option<u64>,

    /// Label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Maximum fee percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxfeepercent: Option<f64>,

    /// Retry for seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_for: Option<u32>,

    /// Maximum delay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxdelay: Option<u32>,

    /// Exempt fee (always pay if fee below this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exemptfee: Option<u64>,
}

/// Keysend params
#[derive(Debug, Clone, Serialize)]
pub struct KeysendParams {
    /// Destination node ID
    pub destination: String,

    /// Amount in millisatoshis
    pub amount_msat: u64,

    /// Label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Maximum fee percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxfeepercent: Option<f64>,

    /// Retry for seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_for: Option<u32>,

    /// Maximum delay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxdelay: Option<u32>,

    /// Exempt fee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exemptfee: Option<u64>,
}

/// Pay response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayResponse {
    /// Payment hash
    pub payment_hash: String,

    /// Payment preimage
    pub payment_preimage: String,

    /// Amount sent in millisatoshis
    pub amount_sent_msat: ClnAmount,

    /// Amount delivered
    pub amount_msat: ClnAmount,

    /// Status
    pub status: String,

    /// Destination
    #[serde(default)]
    pub destination: Option<String>,

    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<f64>,

    /// Parts
    #[serde(default)]
    pub parts: Option<u32>,
}

/// List pays response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPaysResponse {
    pub pays: Vec<ClnPayment>,
}

/// CLN Payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnPayment {
    pub payment_hash: String,
    pub status: String,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub amount_msat: Option<ClnAmount>,
    #[serde(default)]
    pub amount_sent_msat: Option<ClnAmount>,
    #[serde(default)]
    pub created_at: Option<f64>,
    #[serde(default)]
    pub completed_at: Option<f64>,
    #[serde(default)]
    pub preimage: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub bolt11: Option<String>,
}

// ============================================================================
// Invoices
// ============================================================================

/// Create invoice params
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceParams {
    /// Amount in millisatoshis
    pub amount_msat: String,

    /// Label (unique identifier)
    pub label: String,

    /// Description
    pub description: String,

    /// Expiry in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,

    /// Fallback address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<String>>,

    /// Preimage (32 bytes hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimage: Option<String>,
}

/// Invoice response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub payment_hash: String,
    pub bolt11: String,
    pub expires_at: u64,
    #[serde(default)]
    pub payment_secret: Option<String>,
    #[serde(default)]
    pub warning_capacity: Option<String>,
}

/// List invoices response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInvoicesResponse {
    pub invoices: Vec<ClnInvoice>,
}

/// CLN Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnInvoice {
    pub label: String,
    pub payment_hash: String,
    pub status: String,
    pub expires_at: u64,
    #[serde(default)]
    pub bolt11: Option<String>,
    #[serde(default)]
    pub amount_msat: Option<ClnAmount>,
    #[serde(default)]
    pub amount_received_msat: Option<ClnAmount>,
    #[serde(default)]
    pub paid_at: Option<u64>,
    #[serde(default)]
    pub payment_preimage: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

// ============================================================================
// Peers
// ============================================================================

/// Connect params
#[derive(Debug, Clone, Serialize)]
pub struct ConnectParams {
    /// Peer ID
    pub id: String,

    /// Host (optional if in gossip)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Connect response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub id: String,
    pub features: String,
    pub direction: String,
    pub address: ClnAddress,
}

/// List peers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPeersResponse {
    pub peers: Vec<ClnPeer>,
}

/// CLN Peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClnPeer {
    pub id: String,
    pub connected: bool,
    #[serde(default)]
    pub netaddr: Vec<String>,
    #[serde(default)]
    pub features: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cln_amount_msat() {
        let amount = ClnAmount::Msat(1000000);
        assert_eq!(amount.msat(), 1000000);
        assert_eq!(amount.sat(), 1000);

        let amount_str = ClnAmount::String("5000msat".to_string());
        assert_eq!(amount_str.msat(), 5000);
        assert_eq!(amount_str.sat(), 5);
    }

    #[test]
    fn test_cln_info_deserialization() {
        let json = r#"{
            "id": "02abc",
            "alias": "my-cln-node",
            "color": "02abc1",
            "num_peers": 5,
            "num_pending_channels": 0,
            "num_active_channels": 3,
            "num_inactive_channels": 1,
            "network": "bitcoin",
            "blockheight": 800000,
            "version": "v23.11"
        }"#;

        let info: ClnInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "02abc");
        assert_eq!(info.num_active_channels, 3);
    }

    #[test]
    fn test_jsonrpc_request() {
        let request = JsonRpcRequest::new(1, "getinfo", serde_json::json!({}));
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"getinfo\""));
    }
}

