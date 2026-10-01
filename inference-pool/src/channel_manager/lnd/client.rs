//! LND gRPC client implementation.
//!
//! Connects to LND using REST API with macaroon authentication.
//! For production, consider using the full gRPC client with proto files.

use async_trait::async_trait;
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::channel_manager::{
    Channel, ChannelConfig, ChannelEvent, ChannelManager, ChannelState, ChannelStats,
    LightningBackend, PaymentRequest, PaymentResult, PaymentState,
};
use crate::error::{PoolError, PoolResult};

use super::types::*;

// ============================================================================
// LND Configuration
// ============================================================================

/// LND connection configuration
#[derive(Debug, Clone)]
pub struct LndConfig {
    /// LND REST endpoint (e.g., "https://localhost:8080")
    pub rest_endpoint: String,

    /// Path to TLS certificate
    pub cert_path: String,

    /// Path to macaroon file (admin.macaroon)
    pub macaroon_path: String,

    /// Connection timeout in seconds
    pub timeout_secs: u64,

    /// Channel configuration
    pub channel_config: ChannelConfig,
}

impl Default for LndConfig {
    fn default() -> Self {
        Self {
            rest_endpoint: "https://localhost:8080".to_string(),
            cert_path: "~/.lnd/tls.cert".to_string(),
            macaroon_path: "~/.lnd/data/chain/bitcoin/mainnet/admin.macaroon".to_string(),
            timeout_secs: 30,
            channel_config: ChannelConfig::default(),
        }
    }
}

// ============================================================================
// LND Client
// ============================================================================

/// LND REST API client
pub struct LndClient {
    config: LndConfig,
    macaroon: String,
    http_client: reqwest::Client,
}

impl LndClient {
    /// Create a new LND client
    pub async fn connect(config: LndConfig) -> PoolResult<Self> {
        // Read macaroon
        let macaroon = Self::read_macaroon(&config.macaroon_path).await?;

        // Build HTTP client with TLS
        let http_client = Self::build_http_client(&config).await?;

        let client = Self {
            config,
            macaroon,
            http_client,
        };

        // Verify connection
        client.get_info().await?;

        Ok(client)
    }

    /// Read macaroon from file and encode as hex
    async fn read_macaroon(path: &str) -> PoolResult<String> {
        let expanded_path = shellexpand::tilde(path).to_string();
        let bytes = tokio::fs::read(&expanded_path)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to read macaroon: {}", e)))?;
        Ok(hex::encode(bytes))
    }

    /// Build HTTP client with TLS certificate
    async fn build_http_client(config: &LndConfig) -> PoolResult<reqwest::Client> {
        let expanded_cert_path = shellexpand::tilde(&config.cert_path).to_string();

        // Read TLS certificate
        let cert_pem = tokio::fs::read(&expanded_cert_path)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to read TLS cert: {}", e)))?;

        let cert = reqwest::Certificate::from_pem(&cert_pem)
            .map_err(|e| PoolError::internal(format!("Invalid TLS cert: {}", e)))?;

        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true) // For self-signed certs
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| PoolError::internal(format!("Failed to build HTTP client: {}", e)))?;

        Ok(client)
    }

    /// Make authenticated request to LND
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> PoolResult<T> {
        let url = format!("{}{}", self.config.rest_endpoint, path);

        let mut request = self
            .http_client
            .request(method, &url)
            .header("Grpc-Metadata-macaroon", &self.macaroon);

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| PoolError::internal(format!("LND request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(PoolError::internal(format!(
                "LND error {}: {}",
                status, text
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| PoolError::internal(format!("Failed to parse LND response: {}", e)))
    }

    /// Get node info
    pub async fn get_info(&self) -> PoolResult<LndInfo> {
        self.request(reqwest::Method::GET, "/v1/getinfo", None::<&()>)
            .await
    }

    /// Get wallet balance
    pub async fn wallet_balance(&self) -> PoolResult<WalletBalance> {
        self.request(reqwest::Method::GET, "/v1/balance/blockchain", None::<&()>)
            .await
    }

    /// Get channel balance
    pub async fn channel_balance(&self) -> PoolResult<ChannelBalance> {
        self.request(reqwest::Method::GET, "/v1/balance/channels", None::<&()>)
            .await
    }

    /// List channels
    pub async fn list_channels(&self) -> PoolResult<Vec<LndChannel>> {
        let response: ListChannelsResponse = self
            .request(reqwest::Method::GET, "/v1/channels", None::<&()>)
            .await?;
        Ok(response.channels)
    }

    /// List pending channels
    pub async fn pending_channels(&self) -> PoolResult<PendingChannelsResponse> {
        self.request(reqwest::Method::GET, "/v1/channels/pending", None::<&()>)
            .await
    }

    /// Open a channel
    pub async fn open_channel(&self, request: OpenChannelRequest) -> PoolResult<ChannelPoint> {
        self.request(reqwest::Method::POST, "/v1/channels", Some(&request))
            .await
    }

    /// Close a channel
    pub async fn close_channel(&self, channel_point: &str, force: bool) -> PoolResult<String> {
        let parts: Vec<&str> = channel_point.split(':').collect();
        if parts.len() != 2 {
            return Err(PoolError::validation("Invalid channel point format"));
        }

        let path = if force {
            format!(
                "/v1/channels/{}/{}?force=true",
                parts[0], parts[1]
            )
        } else {
            format!("/v1/channels/{}/{}", parts[0], parts[1])
        };

        #[derive(serde::Deserialize)]
        struct CloseResponse {
            closing_txid: Option<String>,
        }

        let response: CloseResponse = self
            .request(reqwest::Method::DELETE, &path, None::<&()>)
            .await?;

        Ok(response.closing_txid.unwrap_or_default())
    }

    /// Connect to a peer
    pub async fn connect_peer(&self, pubkey: &str, host: &str) -> PoolResult<()> {
        let request = ConnectPeerRequest {
            addr: LightningAddress {
                pubkey: pubkey.to_string(),
                host: host.to_string(),
            },
            perm: false,
            timeout: 30,
        };

        #[derive(serde::Deserialize)]
        struct EmptyResponse {}

        let _: EmptyResponse = self
            .request(reqwest::Method::POST, "/v1/peers", Some(&request))
            .await?;

        Ok(())
    }

    /// List peers
    pub async fn list_peers(&self) -> PoolResult<Vec<LndPeer>> {
        let response: ListPeersResponse = self
            .request(reqwest::Method::GET, "/v1/peers", None::<&()>)
            .await?;
        Ok(response.peers)
    }

    /// Send a payment
    pub async fn send_payment(&self, request: SendPaymentRequest) -> PoolResult<SendPaymentResponse> {
        self.request(reqwest::Method::POST, "/v1/channels/transactions", Some(&request))
            .await
    }

    /// Create an invoice
    pub async fn add_invoice(&self, request: AddInvoiceRequest) -> PoolResult<AddInvoiceResponse> {
        self.request(reqwest::Method::POST, "/v1/invoices", Some(&request))
            .await
    }

    /// List payments
    pub async fn list_payments(&self) -> PoolResult<Vec<LndPayment>> {
        let response: ListPaymentsResponse = self
            .request(reqwest::Method::GET, "/v1/payments", None::<&()>)
            .await?;
        Ok(response.payments)
    }
}

// ============================================================================
// LND Channel Manager
// ============================================================================

/// Channel manager implementation using LND
pub struct LndChannelManager {
    client: Arc<LndClient>,
    config: ChannelConfig,
    member_channels: dashmap::DashMap<String, Vec<String>>,
    event_tx: broadcast::Sender<ChannelEvent>,
    node_pubkey: RwLock<String>,
}

impl LndChannelManager {
    /// Create a new LND channel manager
    pub async fn new(lnd_config: LndConfig) -> PoolResult<Self> {
        let channel_config = lnd_config.channel_config.clone();
        let client = LndClient::connect(lnd_config).await?;

        // Get node info
        let info = client.get_info().await?;

        let (event_tx, _) = broadcast::channel(100);

        Ok(Self {
            client: Arc::new(client),
            config: channel_config,
            member_channels: dashmap::DashMap::new(),
            event_tx,
            node_pubkey: RwLock::new(info.identity_pubkey),
        })
    }

    /// Get the node's public key
    pub async fn node_pubkey(&self) -> String {
        self.node_pubkey.read().await.clone()
    }

    /// Convert LND channel to our Channel type
    fn convert_channel(&self, lnd_channel: &LndChannel, member_id: &str) -> Channel {
        let state = if lnd_channel.active {
            ChannelState::Active
        } else {
            ChannelState::Inactive
        };

        let parts: Vec<&str> = lnd_channel.channel_point.split(':').collect();
        let (funding_txid, funding_output) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].parse().unwrap_or(0))
        } else {
            (lnd_channel.channel_point.clone(), 0)
        };

        Channel {
            channel_id: lnd_channel.channel_point.clone(),
            short_channel_id: if lnd_channel.chan_id.is_empty() {
                None
            } else {
                Some(lnd_channel.chan_id.clone())
            },
            peer_pubkey: lnd_channel.remote_pubkey.clone(),
            member_id: member_id.to_string(),
            state,
            capacity_sats: lnd_channel.capacity as u64,
            local_balance_sats: lnd_channel.local_balance as u64,
            remote_balance_sats: lnd_channel.remote_balance as u64,
            reserve_sats: lnd_channel.local_chan_reserve_sat as u64,
            confirmations: lnd_channel.confirmation_height,
            initiator: lnd_channel.initiator,
            private: lnd_channel.private,
            funding_txid,
            funding_output,
            opened_at: Utc::now(), // LND doesn't provide this directly
            last_activity: Utc::now(),
            total_sent_sats: lnd_channel.total_satoshis_sent as u64,
            total_received_sats: lnd_channel.total_satoshis_received as u64,
            htlcs_sent: lnd_channel.num_updates / 2,
            htlcs_received: lnd_channel.num_updates / 2,
        }
    }

    /// Emit a channel event
    fn emit_event(&self, event: ChannelEvent) {
        let _ = self.event_tx.send(event);
    }
}

#[async_trait]
impl ChannelManager for LndChannelManager {
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

        // Open channel via LND
        let request = OpenChannelRequest {
            node_pubkey_string: peer_pubkey.to_string(),
            local_funding_amount: capacity as i64,
            push_sat: self.config.push_amount_sats as i64,
            target_conf: self.config.target_confirmations as i32,
            sat_per_vbyte: self.config.fee_rate_sat_vbyte as i64,
            private: self.config.private_channels,
            min_htlc_msat: self.config.min_htlc_msat as i64,
            remote_csv_delay: 0,
            min_confs: 1,
            spend_unconfirmed: false,
        };

        let channel_point = self.client.open_channel(request).await?;
        let channel_id = channel_point.to_string();

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
            funding_txid: channel_point.funding_txid_str,
            funding_output: channel_point.output_index,
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
        let channels = self.client.list_channels().await?;

        let lnd_channel = channels
            .iter()
            .find(|c| c.channel_point == channel_id)
            .ok_or_else(|| PoolError::not_found(format!("Channel not found: {}", channel_id)))?;

        // Find member ID from our mapping
        let member_id = self
            .member_channels
            .iter()
            .find(|entry| entry.value().contains(&channel_id.to_string()))
            .map(|entry| entry.key().clone())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(self.convert_channel(lnd_channel, &member_id))
    }

    async fn list_channels(&self) -> PoolResult<Vec<Channel>> {
        let lnd_channels = self.client.list_channels().await?;

        let channels: Vec<Channel> = lnd_channels
            .iter()
            .map(|lnd_ch| {
                let member_id = self
                    .member_channels
                    .iter()
                    .find(|entry| entry.value().contains(&lnd_ch.channel_point))
                    .map(|entry| entry.key().clone())
                    .unwrap_or_else(|| "unknown".to_string());

                self.convert_channel(lnd_ch, &member_id)
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
        // Just get fresh data from LND
        self.get_channel(channel_id).await
    }

    async fn send_payment(&self, request: PaymentRequest) -> PoolResult<PaymentResult> {
        let lnd_request = SendPaymentRequest {
            payment_request: String::new(),
            dest: request.destination_pubkey.clone(),
            amt: request.amount_sats as i64,
            payment_hash: request.payment_hash.clone().unwrap_or_default(),
            final_cltv_delta: 40,
            fee_limit_sat: request.max_fee_sats as i64,
            outgoing_chan_id: request.channel_id.clone().unwrap_or_default(),
            timeout_seconds: request.timeout_secs as i32,
            allow_self_payment: false,
            max_parts: 1,
        };

        let response = self.client.send_payment(lnd_request).await?;

        let (state, error) = if !response.payment_error.is_empty() {
            (PaymentState::Failed, Some(response.payment_error))
        } else {
            (PaymentState::Succeeded, None)
        };

        let (fee_sats, hops) = if let Some(ref route) = response.payment_route {
            (route.total_fees_msat / 1000, route.hops.len() as u32)
        } else {
            (0, 1)
        };

        Ok(PaymentResult {
            payment_id: request.payment_id,
            payment_hash: response.payment_hash,
            payment_preimage: if response.payment_preimage.is_empty() {
                None
            } else {
                Some(response.payment_preimage)
            },
            amount_sats: request.amount_sats,
            fee_sats: fee_sats as u64,
            state,
            hops,
            direct_channel: request.channel_id.is_some(),
            channel_id: request.channel_id,
            error,
            timestamp: Utc::now(),
        })
    }

    async fn get_payment(&self, payment_id: &str) -> PoolResult<PaymentResult> {
        let payments = self.client.list_payments().await?;

        let payment = payments
            .iter()
            .find(|p| p.payment_hash == payment_id)
            .ok_or_else(|| PoolError::not_found(format!("Payment not found: {}", payment_id)))?;

        let state = match payment.status.as_str() {
            "SUCCEEDED" => PaymentState::Succeeded,
            "FAILED" => PaymentState::Failed,
            "IN_FLIGHT" => PaymentState::InFlight,
            _ => PaymentState::Pending,
        };

        Ok(PaymentResult {
            payment_id: payment.payment_hash.clone(),
            payment_hash: payment.payment_hash.clone(),
            payment_preimage: if payment.payment_preimage.is_empty() {
                None
            } else {
                Some(payment.payment_preimage.clone())
            },
            amount_sats: payment.value_sat as u64,
            fee_sats: payment.fee_sat as u64,
            state,
            hops: 0, // Not available from list
            direct_channel: false,
            channel_id: None,
            error: if payment.failure_reason.is_empty() {
                None
            } else {
                Some(payment.failure_reason.clone())
            },
            timestamp: chrono::DateTime::from_timestamp(payment.creation_date, 0)
                .unwrap_or_else(Utc::now),
        })
    }

    async fn get_stats(&self) -> PoolResult<ChannelStats> {
        let channels = self.client.list_channels().await?;
        let pending = self.client.pending_channels().await?;

        let mut stats = ChannelStats::default();

        for ch in &channels {
            stats.total_channels += 1;
            stats.total_capacity_sats += ch.capacity as u64;
            stats.total_local_balance_sats += ch.local_balance as u64;
            stats.total_remote_balance_sats += ch.remote_balance as u64;
            stats.total_sent_sats += ch.total_satoshis_sent as u64;
            stats.total_received_sats += ch.total_satoshis_received as u64;

            if ch.active {
                stats.active_channels += 1;
                let reserve = ch.local_chan_reserve_sat as u64;
                stats.total_outbound_capacity_sats +=
                    (ch.local_balance as u64).saturating_sub(reserve);
                stats.total_inbound_capacity_sats +=
                    (ch.remote_balance as u64).saturating_sub(ch.remote_chan_reserve_sat as u64);
            } else {
                stats.inactive_channels += 1;
            }
        }

        stats.pending_channels = pending.pending_open_channels.len() as u32;

        Ok(stats)
    }

    fn subscribe(&self) -> broadcast::Receiver<ChannelEvent> {
        self.event_tx.subscribe()
    }

    fn get_config(&self) -> &ChannelConfig {
        &self.config
    }

    fn backend(&self) -> LightningBackend {
        LightningBackend::Lnd
    }
}

// Add shellexpand as a simple helper
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
    fn test_lnd_config_default() {
        let config = LndConfig::default();
        assert!(config.rest_endpoint.contains("localhost"));
        assert!(config.cert_path.contains("tls.cert"));
    }

    #[test]
    fn test_shellexpand_tilde() {
        std::env::set_var("HOME", "/home/test");
        let expanded = shellexpand::tilde("~/path/to/file");
        assert_eq!(expanded.as_ref(), "/home/test/path/to/file");

        let no_tilde = shellexpand::tilde("/absolute/path");
        assert_eq!(no_tilde.as_ref(), "/absolute/path");
    }
}






