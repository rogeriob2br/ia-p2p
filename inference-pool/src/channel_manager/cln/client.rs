//! CLN (Core Lightning) JSON-RPC client implementation.
//!
//! Connects to CLN using Unix socket with JSON-RPC protocol.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::channel_manager::{
    Channel, ChannelConfig, ChannelEvent, ChannelManager, ChannelState, ChannelStats,
    LightningBackend, PaymentRequest, PaymentResult, PaymentState,
};
use crate::error::{PoolError, PoolResult};

use super::types::*;

// ============================================================================
// CLN Configuration
// ============================================================================

/// CLN connection configuration
#[derive(Debug, Clone)]
pub struct ClnConfig {
    /// Path to lightning-rpc socket
    pub socket_path: String,

    /// Connection timeout in seconds
    pub timeout_secs: u64,

    /// Channel configuration
    pub channel_config: ChannelConfig,
}

impl Default for ClnConfig {
    fn default() -> Self {
        Self {
            socket_path: "~/.lightning/bitcoin/lightning-rpc".to_string(),
            timeout_secs: 30,
            channel_config: ChannelConfig::default(),
        }
    }
}

// ============================================================================
// CLN Client
// ============================================================================

/// CLN JSON-RPC client
pub struct ClnClient {
    config: ClnConfig,
    socket: Mutex<Option<UnixStream>>,
    request_id: AtomicU64,
}

impl ClnClient {
    /// Create a new CLN client
    pub async fn connect(config: ClnConfig) -> PoolResult<Self> {
        let client = Self {
            config,
            socket: Mutex::new(None),
            request_id: AtomicU64::new(1),
        };

        // Verify connection
        client.get_info().await?;

        Ok(client)
    }

    /// Get or create socket connection
    async fn get_socket(&self) -> PoolResult<UnixStream> {
        let expanded_path = shellexpand::tilde(&self.config.socket_path).to_string();

        UnixStream::connect(&expanded_path)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to connect to CLN socket: {}", e)))
    }

    /// Make a JSON-RPC request
    async fn request<T, R>(&self, method: &str, params: T) -> PoolResult<R>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        let request_json = serde_json::to_string(&request)
            .map_err(|e| PoolError::internal(format!("Failed to serialize request: {}", e)))?;

        // Connect and send
        let mut socket = self.get_socket().await?;

        socket
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| PoolError::internal(format!("Failed to send request: {}", e)))?;

        socket
            .flush()
            .await
            .map_err(|e| PoolError::internal(format!("Failed to flush socket: {}", e)))?;

        // Read response
        let mut reader = BufReader::new(socket);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to read response: {}", e)))?;

        let response: JsonRpcResponse<R> = serde_json::from_str(&response_line)
            .map_err(|e| PoolError::internal(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = response.error {
            return Err(PoolError::internal(format!(
                "CLN error {}: {}",
                error.code, error.message
            )));
        }

        response
            .result
            .ok_or_else(|| PoolError::internal("Empty response from CLN".to_string()))
    }

    /// Get node info
    pub async fn get_info(&self) -> PoolResult<ClnInfo> {
        self.request("getinfo", serde_json::json!({})).await
    }

    /// List funds
    pub async fn list_funds(&self) -> PoolResult<ListFundsResponse> {
        self.request("listfunds", serde_json::json!({})).await
    }

    /// List peer channels
    pub async fn list_peer_channels(&self) -> PoolResult<Vec<ClnChannel>> {
        let response: ListPeerChannelsResponse = self
            .request("listpeerchannels", serde_json::json!({}))
            .await?;
        Ok(response.channels)
    }

    /// Open a channel
    pub async fn fund_channel(&self, params: FundChannelParams) -> PoolResult<FundChannelResponse> {
        self.request("fundchannel", params).await
    }

    /// Close a channel
    pub async fn close_channel(&self, id: &str, force: bool) -> PoolResult<CloseResponse> {
        let params = CloseParams {
            id: id.to_string(),
            unilateraltimeout: if force { Some(1) } else { None },
            destination: None,
            fee_negotiation_step: None,
            force_lease_closed: None,
        };
        self.request("close", params).await
    }

    /// Connect to a peer
    pub async fn connect_peer(&self, id: &str, host: Option<&str>, port: Option<u16>) -> PoolResult<ConnectResponse> {
        let params = ConnectParams {
            id: id.to_string(),
            host: host.map(|h| h.to_string()),
            port,
        };
        self.request("connect", params).await
    }

    /// List peers
    pub async fn list_peers(&self) -> PoolResult<Vec<ClnPeer>> {
        let response: ListPeersResponse = self
            .request("listpeers", serde_json::json!({}))
            .await?;
        Ok(response.peers)
    }

    /// Pay an invoice
    pub async fn pay(&self, params: PayParams) -> PoolResult<PayResponse> {
        self.request("pay", params).await
    }

    /// Send a keysend payment
    pub async fn keysend(&self, params: KeysendParams) -> PoolResult<PayResponse> {
        self.request("keysend", params).await
    }

    /// List payments
    pub async fn list_pays(&self) -> PoolResult<Vec<ClnPayment>> {
        let response: ListPaysResponse = self
            .request("listpays", serde_json::json!({}))
            .await?;
        Ok(response.pays)
    }

    /// Create an invoice
    pub async fn create_invoice(&self, params: InvoiceParams) -> PoolResult<InvoiceResponse> {
        self.request("invoice", params).await
    }

    /// List invoices
    pub async fn list_invoices(&self) -> PoolResult<Vec<ClnInvoice>> {
        let response: ListInvoicesResponse = self
            .request("listinvoices", serde_json::json!({}))
            .await?;
        Ok(response.invoices)
    }
}

// ============================================================================
// CLN Channel Manager
// ============================================================================

/// Channel manager implementation using CLN
pub struct ClnChannelManager {
    client: Arc<ClnClient>,
    config: ChannelConfig,
    member_channels: dashmap::DashMap<String, Vec<String>>,
    event_tx: broadcast::Sender<ChannelEvent>,
    node_id: RwLock<String>,
}

impl ClnChannelManager {
    /// Create a new CLN channel manager
    pub async fn new(cln_config: ClnConfig) -> PoolResult<Self> {
        let channel_config = cln_config.channel_config.clone();
        let client = ClnClient::connect(cln_config).await?;

        // Get node info
        let info = client.get_info().await?;

        let (event_tx, _) = broadcast::channel(100);

        Ok(Self {
            client: Arc::new(client),
            config: channel_config,
            member_channels: dashmap::DashMap::new(),
            event_tx,
            node_id: RwLock::new(info.id),
        })
    }

    /// Get the node's public key
    pub async fn node_id(&self) -> String {
        self.node_id.read().await.clone()
    }

    /// Convert CLN channel to our Channel type
    fn convert_channel(&self, cln_channel: &ClnChannel, member_id: &str) -> Channel {
        let state = match cln_channel.state.as_str() {
            "CHANNELD_NORMAL" => {
                if cln_channel.peer_connected {
                    ChannelState::Active
                } else {
                    ChannelState::Inactive
                }
            }
            "CHANNELD_AWAITING_LOCKIN" | "DUALOPEND_AWAITING_LOCKIN" => ChannelState::PendingOpen,
            "CHANNELD_SHUTTING_DOWN" | "CLOSINGD_SIGEXCHANGE" => ChannelState::Closing,
            "CLOSINGD_COMPLETE" | "FUNDING_SPEND_SEEN" => ChannelState::Closed,
            "ONCHAIN" => ChannelState::ForceClosed,
            _ => ChannelState::Opening,
        };

        let channel_id = format!("{}:{}", cln_channel.funding_txid, cln_channel.funding_outnum);
        let capacity = cln_channel.total_msat.sat();
        let local_balance = cln_channel.to_us_msat.sat();
        let remote_balance = cln_channel
            .to_them_msat
            .as_ref()
            .map(|a| a.sat())
            .unwrap_or(capacity.saturating_sub(local_balance));

        Channel {
            channel_id: channel_id.clone(),
            short_channel_id: cln_channel.short_channel_id.clone(),
            peer_pubkey: cln_channel.peer_id.clone(),
            member_id: member_id.to_string(),
            state,
            capacity_sats: capacity,
            local_balance_sats: local_balance,
            remote_balance_sats: remote_balance,
            reserve_sats: cln_channel.our_reserve_msat.as_ref().map(|a| a.sat()).unwrap_or(0),
            confirmations: 0, // CLN doesn't provide this directly
            initiator: cln_channel.opener.as_ref().map(|o| o == "local").unwrap_or(false),
            private: cln_channel.private,
            funding_txid: cln_channel.funding_txid.clone(),
            funding_output: cln_channel.funding_outnum,
            opened_at: Utc::now(),
            last_activity: Utc::now(),
            total_sent_sats: cln_channel.out_fulfilled_msat.as_ref().map(|a| a.sat()).unwrap_or(0),
            total_received_sats: cln_channel.in_fulfilled_msat.as_ref().map(|a| a.sat()).unwrap_or(0),
            htlcs_sent: cln_channel.out_payments_fulfilled.unwrap_or(0),
            htlcs_received: cln_channel.in_payments_fulfilled.unwrap_or(0),
        }
    }

    /// Emit a channel event
    fn emit_event(&self, event: ChannelEvent) {
        let _ = self.event_tx.send(event);
    }
}

#[async_trait]
impl ChannelManager for ClnChannelManager {
    async fn open_channel(
        &self,
        peer_pubkey: &str,
        member_id: &str,
        capacity_sats: Option<u64>,
    ) -> PoolResult<Channel> {
        let capacity = capacity_sats.unwrap_or(self.config.default_capacity_sats);

        // Validate capacity
        if capacity < self.config.min_capacity_sats {
            return Err(PoolError::validation(format!(
                "Capacity {} is below minimum {}",
                capacity, self.config.min_capacity_sats
            )));
        }

        // Open channel via CLN
        let params = FundChannelParams {
            id: peer_pubkey.to_string(),
            amount: format!("{}sat", capacity),
            feerate: Some(format!("{}perkb", self.config.fee_rate_sat_vbyte * 1000)),
            announce: Some(!self.config.private_channels),
            minconf: Some(1),
            push_msat: if self.config.push_amount_sats > 0 {
                Some(self.config.push_amount_sats * 1000)
            } else {
                None
            },
            reserve: None,
        };

        let response = self.client.fund_channel(params).await?;
        let channel_id = format!("{}:{}", response.txid, response.outnum);

        // Store member mapping
        self.member_channels
            .entry(member_id.to_string())
            .or_insert_with(Vec::new)
            .push(channel_id.clone());

        // Create pending channel
        let channel = Channel {
            channel_id: channel_id.clone(),
            short_channel_id: None,
            peer_pubkey: peer_pubkey.to_string(),
            member_id: member_id.to_string(),
            state: ChannelState::PendingOpen,
            capacity_sats: capacity,
            local_balance_sats: capacity - self.config.push_amount_sats,
            remote_balance_sats: self.config.push_amount_sats,
            reserve_sats: (capacity as f64 * self.config.reserve_percent / 100.0) as u64,
            confirmations: 0,
            initiator: true,
            private: self.config.private_channels,
            funding_txid: response.txid,
            funding_output: response.outnum,
            opened_at: Utc::now(),
            last_activity: Utc::now(),
            total_sent_sats: 0,
            total_received_sats: 0,
            htlcs_sent: 0,
            htlcs_received: 0,
        };

        self.emit_event(ChannelEvent::Opened {
            channel_id,
            peer_pubkey: peer_pubkey.to_string(),
            capacity_sats: capacity,
        });

        Ok(channel)
    }

    async fn close_channel(&self, channel_id: &str) -> PoolResult<()> {
        self.client.close_channel(channel_id, false).await?;

        self.emit_event(ChannelEvent::Closing {
            channel_id: channel_id.to_string(),
            cooperative: true,
        });

        Ok(())
    }

    async fn force_close_channel(&self, channel_id: &str) -> PoolResult<()> {
        self.client.close_channel(channel_id, true).await?;

        self.emit_event(ChannelEvent::Closed {
            channel_id: channel_id.to_string(),
            reason: "force_close".to_string(),
        });

        Ok(())
    }

    async fn get_channel(&self, channel_id: &str) -> PoolResult<Channel> {
        let channels = self.client.list_peer_channels().await?;

        let cln_channel = channels
            .iter()
            .find(|c| format!("{}:{}", c.funding_txid, c.funding_outnum) == channel_id)
            .ok_or_else(|| PoolError::not_found(format!("Channel not found: {}", channel_id)))?;

        // Find member ID
        let member_id = self
            .member_channels
            .iter()
            .find(|entry| entry.value().contains(&channel_id.to_string()))
            .map(|entry| entry.key().clone())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(self.convert_channel(cln_channel, &member_id))
    }

    async fn list_channels(&self) -> PoolResult<Vec<Channel>> {
        let cln_channels = self.client.list_peer_channels().await?;

        let channels: Vec<Channel> = cln_channels
            .iter()
            .map(|cln_ch| {
                let channel_id = format!("{}:{}", cln_ch.funding_txid, cln_ch.funding_outnum);
                let member_id = self
                    .member_channels
                    .iter()
                    .find(|entry| entry.value().contains(&channel_id))
                    .map(|entry| entry.key().clone())
                    .unwrap_or_else(|| "unknown".to_string());

                self.convert_channel(cln_ch, &member_id)
            })
            .collect();

        Ok(channels)
    }

    async fn get_member_channels(&self, member_id: &str) -> PoolResult<Vec<Channel>> {
        let channel_ids = self
            .member_channels
            .get(member_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        let all_channels = self.list_channels().await?;

        let member_channels: Vec<Channel> = all_channels
            .into_iter()
            .filter(|ch| channel_ids.contains(&ch.channel_id))
            .collect();

        Ok(member_channels)
    }

    async fn sync_channel(&self, channel_id: &str) -> PoolResult<Channel> {
        self.get_channel(channel_id).await
    }

    async fn send_payment(&self, request: PaymentRequest) -> PoolResult<PaymentResult> {
        // Use keysend for direct payments
        let params = KeysendParams {
            destination: request.destination_pubkey.clone(),
            amount_msat: request.amount_sats * 1000,
            label: Some(request.payment_id.clone()),
            maxfeepercent: Some(1.0), // 1% max fee
            retry_for: Some(request.timeout_secs),
            maxdelay: None,
            exemptfee: Some(request.max_fee_sats * 1000),
        };

        let response = self.client.keysend(params).await?;

        let state = match response.status.as_str() {
            "complete" => PaymentState::Succeeded,
            "failed" => PaymentState::Failed,
            "pending" => PaymentState::InFlight,
            _ => PaymentState::Pending,
        };

        let fee_sats = (response.amount_sent_msat.sat()).saturating_sub(response.amount_msat.sat());

        Ok(PaymentResult {
            payment_id: request.payment_id,
            payment_hash: response.payment_hash,
            payment_preimage: Some(response.payment_preimage),
            amount_sats: response.amount_msat.sat(),
            fee_sats,
            state,
            hops: response.parts.unwrap_or(1),
            direct_channel: request.channel_id.is_some(),
            channel_id: request.channel_id,
            error: None,
            timestamp: Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> PoolResult<PaymentResult> {
        let payments = self.client.list_pays().await?;

        let payment = payments
            .iter()
            .find(|p| p.payment_hash == payment_id || p.label.as_ref() == Some(&payment_id.to_string()))
            .ok_or_else(|| PoolError::not_found(format!("Payment not found: {}", payment_id)))?;

        let state = match payment.status.as_str() {
            "complete" => PaymentState::Succeeded,
            "failed" => PaymentState::Failed,
            "pending" => PaymentState::InFlight,
            _ => PaymentState::Pending,
        };

        let amount_sats = payment.amount_msat.as_ref().map(|a| a.sat()).unwrap_or(0);
        let sent_sats = payment.amount_sent_msat.as_ref().map(|a| a.sat()).unwrap_or(0);

        Ok(PaymentResult {
            payment_id: payment.payment_hash.clone(),
            payment_hash: payment.payment_hash.clone(),
            payment_preimage: payment.preimage.clone(),
            amount_sats,
            fee_sats: sent_sats.saturating_sub(amount_sats),
            state,
            hops: 0,
            direct_channel: false,
            channel_id: None,
            error: None,
            timestamp: payment
                .created_at
                .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
                .unwrap_or_else(Utc::now),
        })
    }

    async fn get_stats(&self) -> PoolResult<ChannelStats> {
        let channels = self.client.list_peer_channels().await?;

        let mut stats = ChannelStats::default();

        for ch in &channels {
            stats.total_channels += 1;
            stats.total_capacity_sats += ch.total_msat.sat();
            stats.total_local_balance_sats += ch.to_us_msat.sat();
            stats.total_remote_balance_sats += ch.to_them_msat.as_ref().map(|a| a.sat()).unwrap_or(0);
            stats.total_sent_sats += ch.out_fulfilled_msat.as_ref().map(|a| a.sat()).unwrap_or(0);
            stats.total_received_sats += ch.in_fulfilled_msat.as_ref().map(|a| a.sat()).unwrap_or(0);

            match ch.state.as_str() {
                "CHANNELD_NORMAL" => {
                    if ch.peer_connected {
                        stats.active_channels += 1;
                        stats.total_outbound_capacity_sats +=
                            ch.spendable_msat.as_ref().map(|a| a.sat()).unwrap_or(0);
                        stats.total_inbound_capacity_sats +=
                            ch.receivable_msat.as_ref().map(|a| a.sat()).unwrap_or(0);
                    } else {
                        stats.inactive_channels += 1;
                    }
                }
                "CHANNELD_AWAITING_LOCKIN" | "DUALOPEND_AWAITING_LOCKIN" => {
                    stats.pending_channels += 1;
                }
                _ => {}
            }
        }

        Ok(stats)
    }

    fn subscribe(&self) -> broadcast::Receiver<ChannelEvent> {
        self.event_tx.subscribe()
    }

    fn get_config(&self) -> &ChannelConfig {
        &self.config
    }

    fn backend(&self) -> LightningBackend {
        LightningBackend::Cln
    }
}

// Shell expand helper
mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<str> {
        if path.starts_with("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return std::borrow::Cow::Owned(format!(
                    "{}{}",
                    home.to_string_lossy(),
                    &path[1..]
                ));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cln_config_default() {
        let config = ClnConfig::default();
        assert!(config.socket_path.contains("lightning-rpc"));
    }

    #[test]
    fn test_shellexpand_tilde() {
        std::env::set_var("HOME", "/home/test");
        let expanded = shellexpand::tilde("~/path/to/file");
        assert_eq!(expanded.as_ref(), "/home/test/path/to/file");
    }
}






