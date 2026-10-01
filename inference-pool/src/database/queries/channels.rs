//! Lightning channel queries.

use chrono::{DateTime, Utc};
use sqlx::{Any, Row};

use crate::channel_manager::{Channel, ChannelState};
use crate::error::{PoolError, PoolResult};

/// Queries for lightning channels
pub struct ChannelQueries;

impl ChannelQueries {
    /// Insert or update a channel
    pub async fn upsert(pool: &sqlx::Pool<Any>, channel: &Channel) -> PoolResult<()> {
        let status = channel_state_to_str(&channel.state);

        sqlx::query(
            r#"
            INSERT INTO lightning_channels (
                channel_id, member_id, node_pubkey, capacity_sats,
                local_balance_sats, remote_balance_sats, status, private_channel,
                opened_at, last_payment_at, total_payments, total_sats_sent,
                total_sats_received, funding_txid, funding_output
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(channel_id) DO UPDATE SET
                local_balance_sats = excluded.local_balance_sats,
                remote_balance_sats = excluded.remote_balance_sats,
                status = excluded.status,
                last_payment_at = excluded.last_payment_at,
                total_payments = excluded.total_payments,
                total_sats_sent = excluded.total_sats_sent,
                total_sats_received = excluded.total_sats_received
            "#,
        )
        .bind(&channel.channel_id)
        .bind(&channel.member_id)
        .bind(&channel.peer_pubkey)
        .bind(channel.capacity_sats as i64)
        .bind(channel.local_balance_sats as i64)
        .bind(channel.remote_balance_sats as i64)
        .bind(status)
        .bind(channel.private)
        .bind(channel.opened_at.to_rfc3339())
        .bind(channel.last_activity.to_rfc3339())
        .bind(channel.htlcs_sent as i64)
        .bind(channel.total_sent_sats as i64)
        .bind(channel.total_received_sats as i64)
        .bind(&channel.funding_txid)
        .bind(channel.funding_output as i64)
        .execute(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to upsert channel: {}", e)))?;

        Ok(())
    }

    /// Get a channel by ID
    pub async fn get_by_id(pool: &sqlx::Pool<Any>, channel_id: &str) -> PoolResult<Option<Channel>> {
        let row = sqlx::query("SELECT * FROM lightning_channels WHERE channel_id = ?")
            .bind(channel_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get channel: {}", e)))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_channel(&row)?)),
            None => Ok(None),
        }
    }

    /// Get all channels
    pub async fn get_all(pool: &sqlx::Pool<Any>) -> PoolResult<Vec<Channel>> {
        let rows = sqlx::query("SELECT * FROM lightning_channels ORDER BY opened_at")
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get channels: {}", e)))?;

        let channels: Result<Vec<_>, _> = rows.iter().map(Self::row_to_channel).collect();
        channels
    }

    /// Get channels by member
    pub async fn get_by_member(pool: &sqlx::Pool<Any>, member_id: &str) -> PoolResult<Vec<Channel>> {
        let rows = sqlx::query("SELECT * FROM lightning_channels WHERE member_id = ?")
            .bind(member_id)
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get channels by member: {}", e)))?;

        let channels: Result<Vec<_>, _> = rows.iter().map(Self::row_to_channel).collect();
        channels
    }

    /// Get active channels
    pub async fn get_active(pool: &sqlx::Pool<Any>) -> PoolResult<Vec<Channel>> {
        let rows = sqlx::query("SELECT * FROM lightning_channels WHERE status = 'active'")
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get active channels: {}", e)))?;

        let channels: Result<Vec<_>, _> = rows.iter().map(Self::row_to_channel).collect();
        channels
    }

    /// Update channel status
    pub async fn update_status(
        pool: &sqlx::Pool<Any>,
        channel_id: &str,
        state: &ChannelState,
    ) -> PoolResult<()> {
        sqlx::query("UPDATE lightning_channels SET status = ? WHERE channel_id = ?")
            .bind(channel_state_to_str(state))
            .bind(channel_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update channel status: {}", e)))?;

        Ok(())
    }

    /// Update channel balances after a payment
    pub async fn update_balances(
        pool: &sqlx::Pool<Any>,
        channel_id: &str,
        local_balance: u64,
        remote_balance: u64,
        payment_amount: u64,
        sent: bool,
    ) -> PoolResult<()> {
        let (sent_delta, received_delta) = if sent {
            (payment_amount as i64, 0i64)
        } else {
            (0i64, payment_amount as i64)
        };

        sqlx::query(
            r#"
            UPDATE lightning_channels SET
                local_balance_sats = ?,
                remote_balance_sats = ?,
                last_payment_at = ?,
                total_payments = total_payments + 1,
                total_sats_sent = total_sats_sent + ?,
                total_sats_received = total_sats_received + ?
            WHERE channel_id = ?
            "#,
        )
        .bind(local_balance as i64)
        .bind(remote_balance as i64)
        .bind(Utc::now().to_rfc3339())
        .bind(sent_delta)
        .bind(received_delta)
        .bind(channel_id)
        .execute(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to update balances: {}", e)))?;

        Ok(())
    }

    /// Delete a channel
    pub async fn delete(pool: &sqlx::Pool<Any>, channel_id: &str) -> PoolResult<bool> {
        let result = sqlx::query("DELETE FROM lightning_channels WHERE channel_id = ?")
            .bind(channel_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to delete channel: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Get total capacity and balance statistics
    pub async fn get_stats(pool: &sqlx::Pool<Any>) -> PoolResult<ChannelStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_channels,
                SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active_channels,
                COALESCE(SUM(capacity_sats), 0) as total_capacity,
                COALESCE(SUM(local_balance_sats), 0) as total_local,
                COALESCE(SUM(remote_balance_sats), 0) as total_remote,
                COALESCE(SUM(total_sats_sent), 0) as total_sent,
                COALESCE(SUM(total_sats_received), 0) as total_received,
                COALESCE(SUM(total_payments), 0) as total_payments
            FROM lightning_channels
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to get channel stats: {}", e)))?;

        Ok(ChannelStats {
            total_channels: row.try_get::<i64, _>("total_channels").unwrap_or(0) as u32,
            active_channels: row.try_get::<i64, _>("active_channels").unwrap_or(0) as u32,
            total_capacity_sats: row.try_get::<i64, _>("total_capacity").unwrap_or(0) as u64,
            total_local_balance_sats: row.try_get::<i64, _>("total_local").unwrap_or(0) as u64,
            total_remote_balance_sats: row.try_get::<i64, _>("total_remote").unwrap_or(0) as u64,
            total_sent_sats: row.try_get::<i64, _>("total_sent").unwrap_or(0) as u64,
            total_received_sats: row.try_get::<i64, _>("total_received").unwrap_or(0) as u64,
            total_payments: row.try_get::<i64, _>("total_payments").unwrap_or(0) as u64,
        })
    }

    fn row_to_channel(row: &sqlx::any::AnyRow) -> PoolResult<Channel> {
        let opened_at_str: String = row.try_get("opened_at").unwrap_or_default();
        let last_payment_str: Option<String> = row.try_get("last_payment_at").ok();
        let status_str: String = row.try_get("status").unwrap_or_default();

        let opened_at = DateTime::parse_from_rfc3339(&opened_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let last_activity = last_payment_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(opened_at);

        Ok(Channel {
            channel_id: row.try_get("channel_id").unwrap_or_default(),
            short_channel_id: None,
            peer_pubkey: row.try_get("node_pubkey").unwrap_or_default(),
            member_id: row.try_get("member_id").unwrap_or_default(),
            state: str_to_channel_state(&status_str),
            capacity_sats: row.try_get::<i64, _>("capacity_sats").unwrap_or(0) as u64,
            local_balance_sats: row.try_get::<i64, _>("local_balance_sats").unwrap_or(0) as u64,
            remote_balance_sats: row.try_get::<i64, _>("remote_balance_sats").unwrap_or(0) as u64,
            reserve_sats: 0,
            confirmations: 0,
            initiator: true,
            private: row.try_get::<i32, _>("private_channel").unwrap_or(0) == 1,
            funding_txid: row.try_get("funding_txid").unwrap_or_default(),
            funding_output: row.try_get::<i64, _>("funding_output").unwrap_or(0) as u32,
            opened_at,
            last_activity,
            total_sent_sats: row.try_get::<i64, _>("total_sats_sent").unwrap_or(0) as u64,
            total_received_sats: row.try_get::<i64, _>("total_sats_received").unwrap_or(0) as u64,
            htlcs_sent: row.try_get::<i64, _>("total_payments").unwrap_or(0) as u64,
            htlcs_received: 0,
        })
    }
}

/// Channel statistics
#[derive(Debug, Clone, Default)]
pub struct ChannelStats {
    pub total_channels: u32,
    pub active_channels: u32,
    pub total_capacity_sats: u64,
    pub total_local_balance_sats: u64,
    pub total_remote_balance_sats: u64,
    pub total_sent_sats: u64,
    pub total_received_sats: u64,
    pub total_payments: u64,
}

fn channel_state_to_str(state: &ChannelState) -> &'static str {
    match state {
        ChannelState::Opening => "opening",
        ChannelState::PendingOpen => "pending_open",
        ChannelState::Active => "active",
        ChannelState::Inactive => "inactive",
        ChannelState::Closing => "closing",
        ChannelState::PendingClose => "pending_close",
        ChannelState::Closed => "closed",
        ChannelState::ForceClosed => "force_closed",
    }
}

fn str_to_channel_state(s: &str) -> ChannelState {
    match s {
        "opening" => ChannelState::Opening,
        "pending_open" => ChannelState::PendingOpen,
        "active" => ChannelState::Active,
        "inactive" => ChannelState::Inactive,
        "closing" => ChannelState::Closing,
        "pending_close" => ChannelState::PendingClose,
        "closed" => ChannelState::Closed,
        "force_closed" => ChannelState::ForceClosed,
        _ => ChannelState::Inactive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{DatabaseConfig, DatabasePool, MigrationManager};

    async fn setup_db() -> sqlx::Pool<Any> {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);
        manager.migrate_up().await.unwrap();
        manager.pool.pool().clone()
    }

    fn create_test_channel() -> Channel {
        Channel {
            channel_id: "tx123:0".to_string(),
            short_channel_id: Some("123x456x0".to_string()),
            peer_pubkey: "02abc".to_string(),
            member_id: "member-1".to_string(),
            state: ChannelState::Active,
            capacity_sats: 1_000_000,
            local_balance_sats: 500_000,
            remote_balance_sats: 500_000,
            reserve_sats: 10_000,
            confirmations: 6,
            initiator: true,
            private: false,
            funding_txid: "tx123".to_string(),
            funding_output: 0,
            opened_at: Utc::now(),
            last_activity: Utc::now(),
            total_sent_sats: 0,
            total_received_sats: 0,
            htlcs_sent: 0,
            htlcs_received: 0,
        }
    }

    #[tokio::test]
    async fn test_upsert_and_get() {
        let pool = setup_db().await;
        let channel = create_test_channel();

        ChannelQueries::upsert(&pool, &channel).await.unwrap();

        let loaded = ChannelQueries::get_by_id(&pool, "tx123:0").await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.capacity_sats, 1_000_000);
        assert_eq!(loaded.state, ChannelState::Active);
    }

    #[tokio::test]
    async fn test_update_balances() {
        let pool = setup_db().await;
        ChannelQueries::upsert(&pool, &create_test_channel()).await.unwrap();

        ChannelQueries::update_balances(&pool, "tx123:0", 400_000, 600_000, 100_000, true)
            .await
            .unwrap();

        let loaded = ChannelQueries::get_by_id(&pool, "tx123:0").await.unwrap().unwrap();
        assert_eq!(loaded.local_balance_sats, 400_000);
        assert_eq!(loaded.remote_balance_sats, 600_000);
        assert_eq!(loaded.total_sent_sats, 100_000);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let pool = setup_db().await;
        ChannelQueries::upsert(&pool, &create_test_channel()).await.unwrap();

        let stats = ChannelQueries::get_stats(&pool).await.unwrap();
        assert_eq!(stats.total_channels, 1);
        assert_eq!(stats.active_channels, 1);
        assert_eq!(stats.total_capacity_sats, 1_000_000);
    }
}

