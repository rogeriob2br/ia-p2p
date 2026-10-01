//! Lightning Channel Manager implementation.

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::error::{PoolError, PoolResult};

use super::types::{
    Channel, ChannelConfig, ChannelEvent, ChannelState, ChannelStats, PaymentRequest,
    PaymentResult, PaymentState,
};

// ============================================================================
// Lightning Backend
// ============================================================================

/// Lightning backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningBackend {
    /// LND (Lightning Network Daemon)
    Lnd,
    /// CLN (Core Lightning / c-lightning)
    Cln,
    /// Mock backend for testing
    Mock,
}

impl Default for LightningBackend {
    fn default() -> Self {
        Self::Mock
    }
}

// ============================================================================
// Channel Manager Trait
// ============================================================================

/// Channel Manager trait for Lightning operations
#[async_trait]
pub trait ChannelManager: Send + Sync {
    // Channel Operations
    /// Open a channel with a peer
    async fn open_channel(
        &self,
        peer_pubkey: &str,
        member_id: &str,
        capacity_sats: Option<u64>,
    ) -> PoolResult<Channel>;

    /// Close a channel cooperatively
    async fn close_channel(&self, channel_id: &str) -> PoolResult<()>;

    /// Force close a channel
    async fn force_close_channel(&self, channel_id: &str) -> PoolResult<()>;

    /// Get a channel by ID
    async fn get_channel(&self, channel_id: &str) -> PoolResult<Channel>;

    /// Get all channels
    async fn list_channels(&self) -> PoolResult<Vec<Channel>>;

    /// Get channels for a member
    async fn get_member_channels(&self, member_id: &str) -> PoolResult<Vec<Channel>>;

    /// Update channel state from Lightning node
    async fn sync_channel(&self, channel_id: &str) -> PoolResult<Channel>;

    // Payment Operations
    /// Send a payment via direct channel or routing
    async fn send_payment(&self, request: PaymentRequest) -> PoolResult<PaymentResult>;

    /// Get payment status
    async fn get_payment(&self, payment_id: &str) -> PoolResult<PaymentResult>;

    // Stats and Monitoring
    /// Get aggregated channel statistics
    async fn get_stats(&self) -> PoolResult<ChannelStats>;

    /// Subscribe to channel events
    fn subscribe(&self) -> broadcast::Receiver<ChannelEvent>;

    // Configuration
    /// Get channel configuration
    fn get_config(&self) -> &ChannelConfig;

    /// Get Lightning backend type
    fn backend(&self) -> LightningBackend;
}

// ============================================================================
// In-Memory Implementation (Mock)
// ============================================================================

/// In-memory channel manager for testing
pub struct InMemoryChannelManager {
    config: ChannelConfig,
    channels: DashMap<String, Channel>,
    member_channels: DashMap<String, Vec<String>>,
    payments: DashMap<String, PaymentResult>,
    channel_counter: Arc<RwLock<u64>>,
    payment_counter: Arc<RwLock<u64>>,
    event_tx: broadcast::Sender<ChannelEvent>,
}

impl InMemoryChannelManager {
    /// Create a new in-memory channel manager
    pub fn new(config: ChannelConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            config,
            channels: DashMap::new(),
            member_channels: DashMap::new(),
            payments: DashMap::new(),
            channel_counter: Arc::new(RwLock::new(0)),
            payment_counter: Arc::new(RwLock::new(0)),
            event_tx,
        }
    }

    /// Generate a mock channel ID
    async fn next_channel_id(&self) -> String {
        let mut counter = self.channel_counter.write().await;
        *counter += 1;
        format!("channel-mock-{:08x}:0", *counter)
    }

    /// Generate a mock payment ID
    async fn next_payment_id(&self) -> String {
        let mut counter = self.payment_counter.write().await;
        *counter += 1;
        format!("payment-{:08}", *counter)
    }

    /// Generate a mock payment hash
    fn generate_payment_hash() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        hex::encode(bytes)
    }

    /// Emit a channel event
    fn emit_event(&self, event: ChannelEvent) {
        let _ = self.event_tx.send(event);
    }
}

#[async_trait]
impl ChannelManager for InMemoryChannelManager {
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
        if capacity > self.config.max_capacity_sats {
            return Err(PoolError::validation(format!(
                "Capacity {} exceeds maximum {}",
                capacity, self.config.max_capacity_sats
            )));
        }

        let channel_id = self.next_channel_id().await;
        let reserve = (capacity as f64 * self.config.reserve_percent / 100.0) as u64;
        let now = Utc::now();

        let channel = Channel {
            channel_id: channel_id.clone(),
            short_channel_id: None,
            peer_pubkey: peer_pubkey.to_string(),
            member_id: member_id.to_string(),
            state: ChannelState::PendingOpen,
            capacity_sats: capacity,
            local_balance_sats: capacity - self.config.push_amount_sats,
            remote_balance_sats: self.config.push_amount_sats,
            reserve_sats: reserve,
            confirmations: 0,
            initiator: true,
            private: self.config.private_channels,
            funding_txid: format!("mock-funding-{}", channel_id),
            funding_output: 0,
            opened_at: now,
            last_activity: now,
            total_sent_sats: 0,
            total_received_sats: 0,
            htlcs_sent: 0,
            htlcs_received: 0,
        };

        // Store channel
        self.channels.insert(channel_id.clone(), channel.clone());

        // Map to member
        self.member_channels
            .entry(member_id.to_string())
            .or_insert_with(Vec::new)
            .push(channel_id.clone());

        // Emit event
        self.emit_event(ChannelEvent::Opened {
            channel_id: channel_id.clone(),
            peer_pubkey: peer_pubkey.to_string(),
            capacity_sats: capacity,
        });

        Ok(channel)
    }

    async fn close_channel(&self, channel_id: &str) -> PoolResult<()> {
        let mut channel = self.get_channel(channel_id).await?;

        if channel.state.is_terminal() {
            return Err(PoolError::validation("Channel already closed"));
        }

        channel.state = ChannelState::Closing;
        self.channels.insert(channel_id.to_string(), channel);

        self.emit_event(ChannelEvent::Closing {
            channel_id: channel_id.to_string(),
            cooperative: true,
        });

        // Simulate closure completion
        if let Some(mut ch) = self.channels.get_mut(channel_id) {
            ch.state = ChannelState::Closed;
        }

        self.emit_event(ChannelEvent::Closed {
            channel_id: channel_id.to_string(),
            reason: "cooperative_close".to_string(),
        });

        Ok(())
    }

    async fn force_close_channel(&self, channel_id: &str) -> PoolResult<()> {
        let mut channel = self.get_channel(channel_id).await?;

        if channel.state.is_terminal() {
            return Err(PoolError::validation("Channel already closed"));
        }

        channel.state = ChannelState::ForceClosed;
        self.channels.insert(channel_id.to_string(), channel);

        self.emit_event(ChannelEvent::Closed {
            channel_id: channel_id.to_string(),
            reason: "force_close".to_string(),
        });

        Ok(())
    }

    async fn get_channel(&self, channel_id: &str) -> PoolResult<Channel> {
        self.channels
            .get(channel_id)
            .map(|c| c.clone())
            .ok_or_else(|| PoolError::not_found(format!("Channel not found: {}", channel_id)))
    }

    async fn list_channels(&self) -> PoolResult<Vec<Channel>> {
        Ok(self.channels.iter().map(|c| c.clone()).collect())
    }

    async fn get_member_channels(&self, member_id: &str) -> PoolResult<Vec<Channel>> {
        let channel_ids = self
            .member_channels
            .get(member_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        let channels: Vec<Channel> = channel_ids
            .iter()
            .filter_map(|id| self.channels.get(id).map(|c| c.clone()))
            .collect();

        Ok(channels)
    }

    async fn sync_channel(&self, channel_id: &str) -> PoolResult<Channel> {
        let mut channel = self.get_channel(channel_id).await?;

        // Simulate sync: if pending, move to active after "confirmations"
        if channel.state == ChannelState::PendingOpen {
            channel.confirmations += 1;
            if channel.confirmations >= self.config.target_confirmations {
                channel.state = ChannelState::Active;
                channel.short_channel_id =
                    Some(format!("{}x{}x{}", 800000 + channel.confirmations, 1, 0));

                self.emit_event(ChannelEvent::Active {
                    channel_id: channel_id.to_string(),
                });
            }
            self.channels.insert(channel_id.to_string(), channel.clone());
        }

        Ok(channel)
    }

    async fn send_payment(&self, request: PaymentRequest) -> PoolResult<PaymentResult> {
        // Find a suitable channel
        let channel = if let Some(ref channel_id) = request.channel_id {
            // Use specific channel
            let ch = self.get_channel(channel_id).await?;
            if !ch.can_send(request.amount_sats) {
                return Err(PoolError::payment(format!(
                    "Channel {} cannot send {} sats",
                    channel_id, request.amount_sats
                )));
            }
            Some(ch)
        } else if let Some(ref member_id) = request.member_id {
            // Find channel for member
            let channels = self.get_member_channels(member_id).await?;
            channels
                .into_iter()
                .find(|c| c.can_send(request.amount_sats))
        } else {
            // Find any suitable channel
            self.channels
                .iter()
                .find(|c| c.peer_pubkey == request.destination_pubkey && c.can_send(request.amount_sats))
                .map(|c| c.clone())
        };

        let payment_hash = request
            .payment_hash
            .clone()
            .unwrap_or_else(Self::generate_payment_hash);

        let (direct_channel, channel_id, fee_sats, hops) = if let Some(ch) = channel {
            // Direct channel payment - update balances
            if let Some(mut channel) = self.channels.get_mut(&ch.channel_id) {
                channel.local_balance_sats -= request.amount_sats;
                channel.remote_balance_sats += request.amount_sats;
                channel.total_sent_sats += request.amount_sats;
                channel.htlcs_sent += 1;
                channel.last_activity = Utc::now();
            }

            self.emit_event(ChannelEvent::PaymentSent {
                channel_id: ch.channel_id.clone(),
                amount_sats: request.amount_sats,
                payment_hash: payment_hash.clone(),
            });

            (true, Some(ch.channel_id), 0, 1)
        } else {
            // Simulate routing (mock)
            let fee = request.amount_sats / 1000; // 0.1% fee
            if fee > request.max_fee_sats {
                let result = PaymentResult {
                    payment_id: request.payment_id.clone(),
                    payment_hash,
                    payment_preimage: None,
                    amount_sats: request.amount_sats,
                    fee_sats: 0,
                    state: PaymentState::Failed,
                    hops: 0,
                    direct_channel: false,
                    channel_id: None,
                    error: Some("Fee exceeds maximum".to_string()),
                    timestamp: Utc::now(),
                };
                self.payments.insert(request.payment_id.clone(), result.clone());
                return Ok(result);
            }
            (false, None, fee, 3)
        };

        // Generate preimage (mock)
        let preimage = Self::generate_payment_hash();

        let result = PaymentResult {
            payment_id: request.payment_id.clone(),
            payment_hash,
            payment_preimage: Some(preimage),
            amount_sats: request.amount_sats,
            fee_sats,
            state: PaymentState::Succeeded,
            hops,
            direct_channel,
            channel_id,
            error: None,
            timestamp: Utc::now(),
        };

        self.payments.insert(request.payment_id.clone(), result.clone());

        Ok(result)
    }

    async fn get_payment(&self, payment_id: &str) -> PoolResult<PaymentResult> {
        self.payments
            .get(payment_id)
            .map(|p| p.clone())
            .ok_or_else(|| PoolError::not_found(format!("Payment not found: {}", payment_id)))
    }

    async fn get_stats(&self) -> PoolResult<ChannelStats> {
        let mut stats = ChannelStats::default();

        for channel in self.channels.iter() {
            stats.total_channels += 1;
            stats.total_capacity_sats += channel.capacity_sats;
            stats.total_local_balance_sats += channel.local_balance_sats;
            stats.total_remote_balance_sats += channel.remote_balance_sats;
            stats.total_sent_sats += channel.total_sent_sats;
            stats.total_received_sats += channel.total_received_sats;

            match channel.state {
                ChannelState::Active => {
                    stats.active_channels += 1;
                    stats.total_outbound_capacity_sats += channel.outbound_capacity();
                    stats.total_inbound_capacity_sats += channel.inbound_capacity();
                }
                ChannelState::Opening | ChannelState::PendingOpen => {
                    stats.pending_channels += 1;
                }
                ChannelState::Inactive => {
                    stats.inactive_channels += 1;
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
        LightningBackend::Mock
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_manager() -> InMemoryChannelManager {
        InMemoryChannelManager::new(ChannelConfig::default())
    }

    #[tokio::test]
    async fn test_open_channel() {
        let manager = create_manager();

        let channel = manager
            .open_channel("02abc123", "member-1", Some(500_000))
            .await
            .unwrap();

        assert_eq!(channel.peer_pubkey, "02abc123");
        assert_eq!(channel.member_id, "member-1");
        assert_eq!(channel.capacity_sats, 500_000);
        assert_eq!(channel.state, ChannelState::PendingOpen);
    }

    #[tokio::test]
    async fn test_channel_lifecycle() {
        let manager = create_manager();

        // Open
        let channel = manager
            .open_channel("02abc123", "member-1", None)
            .await
            .unwrap();

        assert_eq!(channel.state, ChannelState::PendingOpen);

        // Sync to get confirmations
        for _ in 0..3 {
            manager.sync_channel(&channel.channel_id).await.unwrap();
        }

        let synced = manager.get_channel(&channel.channel_id).await.unwrap();
        assert_eq!(synced.state, ChannelState::Active);
        assert!(synced.short_channel_id.is_some());

        // Close
        manager.close_channel(&channel.channel_id).await.unwrap();
        let closed = manager.get_channel(&channel.channel_id).await.unwrap();
        assert_eq!(closed.state, ChannelState::Closed);
    }

    #[tokio::test]
    async fn test_payment_via_direct_channel() {
        let manager = create_manager();

        // Open and activate channel
        let channel = manager
            .open_channel("02abc123", "member-1", Some(1_000_000))
            .await
            .unwrap();

        for _ in 0..3 {
            manager.sync_channel(&channel.channel_id).await.unwrap();
        }

        // Make payment
        let request = PaymentRequest {
            payment_id: "pay-1".to_string(),
            destination_pubkey: "02abc123".to_string(),
            amount_sats: 10_000,
            payment_hash: None,
            description: Some("Test payment".to_string()),
            timeout_secs: 60,
            max_fee_sats: 100,
            channel_id: Some(channel.channel_id.clone()),
            member_id: None,
        };

        let result = manager.send_payment(request).await.unwrap();

        assert_eq!(result.state, PaymentState::Succeeded);
        assert!(result.direct_channel);
        assert_eq!(result.fee_sats, 0);
        assert!(result.payment_preimage.is_some());

        // Check channel balance updated
        let updated = manager.get_channel(&channel.channel_id).await.unwrap();
        assert_eq!(updated.total_sent_sats, 10_000);
    }

    #[tokio::test]
    async fn test_member_channels() {
        let manager = create_manager();

        // Open multiple channels for a member
        manager
            .open_channel("02abc123", "member-1", None)
            .await
            .unwrap();
        manager
            .open_channel("02def456", "member-1", None)
            .await
            .unwrap();
        manager
            .open_channel("02ghi789", "member-2", None)
            .await
            .unwrap();

        let member1_channels = manager.get_member_channels("member-1").await.unwrap();
        let member2_channels = manager.get_member_channels("member-2").await.unwrap();

        assert_eq!(member1_channels.len(), 2);
        assert_eq!(member2_channels.len(), 1);
    }

    #[tokio::test]
    async fn test_channel_stats() {
        let manager = create_manager();

        // Open channels
        let ch1 = manager
            .open_channel("02abc123", "member-1", Some(500_000))
            .await
            .unwrap();
        manager
            .open_channel("02def456", "member-2", Some(1_000_000))
            .await
            .unwrap();

        // Activate first channel
        for _ in 0..3 {
            manager.sync_channel(&ch1.channel_id).await.unwrap();
        }

        let stats = manager.get_stats().await.unwrap();

        assert_eq!(stats.total_channels, 2);
        assert_eq!(stats.active_channels, 1);
        assert_eq!(stats.pending_channels, 1);
        assert_eq!(stats.total_capacity_sats, 1_500_000);
    }

    #[tokio::test]
    async fn test_capacity_validation() {
        let manager = create_manager();

        // Too small
        let result = manager
            .open_channel("02abc123", "member-1", Some(1000))
            .await;
        assert!(result.is_err());

        // Too large
        let result = manager
            .open_channel("02abc123", "member-1", Some(1_000_000_000))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let manager = create_manager();
        let mut rx = manager.subscribe();

        manager
            .open_channel("02abc123", "member-1", None)
            .await
            .unwrap();

        // Should receive Opened event
        let event = rx.try_recv().unwrap();
        match event {
            ChannelEvent::Opened { peer_pubkey, .. } => {
                assert_eq!(peer_pubkey, "02abc123");
            }
            _ => panic!("Expected Opened event"),
        }
    }
}






