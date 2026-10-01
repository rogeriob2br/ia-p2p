//! Pool member queries.

use chrono::{DateTime, Utc};
use sqlx::{Any, Row};

use crate::error::{PoolError, PoolResult};
use crate::models::{
    MemberMetrics, MemberStatus, ModelInfo, NodeCapabilities, PoolMember,
};

/// Queries for pool members
pub struct MemberQueries;

impl MemberQueries {
    /// Insert a new member
    pub async fn insert(pool: &sqlx::Pool<Any>, member: &PoolMember) -> PoolResult<()> {
        let capabilities_json = serde_json::to_string(&member.capabilities)
            .map_err(|e| PoolError::internal(format!("Failed to serialize capabilities: {}", e)))?;

        let models_json = serde_json::to_string(&member.models)
            .map_err(|e| PoolError::internal(format!("Failed to serialize models: {}", e)))?;

        let multiaddrs_json = serde_json::to_string(&member.multiaddrs)
            .map_err(|e| PoolError::internal(format!("Failed to serialize multiaddrs: {}", e)))?;

        let metrics_json = serde_json::to_string(&member.metrics)
            .map_err(|e| PoolError::internal(format!("Failed to serialize metrics: {}", e)))?;

        let status_str = status_to_str(&member.status);

        sqlx::query(
            r#"
            INSERT INTO pool_members (
                node_id, peer_id, capabilities, models, status,
                lightning_pubkey, multiaddrs, registered_at, last_heartbeat, metrics
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(node_id) DO UPDATE SET
                peer_id = excluded.peer_id,
                capabilities = excluded.capabilities,
                models = excluded.models,
                status = excluded.status,
                lightning_pubkey = excluded.lightning_pubkey,
                multiaddrs = excluded.multiaddrs,
                last_heartbeat = excluded.last_heartbeat,
                metrics = excluded.metrics
            "#,
        )
        .bind(&member.node_id)
        .bind(&member.peer_id)
        .bind(&capabilities_json)
        .bind(&models_json)
        .bind(status_str)
        .bind(&member.lightning_pubkey)
        .bind(&multiaddrs_json)
        .bind(member.registered_at.to_rfc3339())
        .bind(member.last_heartbeat.to_rfc3339())
        .bind(&metrics_json)
        .execute(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to insert member: {}", e)))?;

        Ok(())
    }

    /// Get a member by node_id
    pub async fn get_by_id(pool: &sqlx::Pool<Any>, node_id: &str) -> PoolResult<Option<PoolMember>> {
        let row = sqlx::query("SELECT * FROM pool_members WHERE node_id = ?")
            .bind(node_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get member: {}", e)))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_member(&row)?)),
            None => Ok(None),
        }
    }

    /// Get all members
    pub async fn get_all(pool: &sqlx::Pool<Any>) -> PoolResult<Vec<PoolMember>> {
        let rows = sqlx::query("SELECT * FROM pool_members ORDER BY registered_at")
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get members: {}", e)))?;

        let members: Result<Vec<_>, _> = rows.iter().map(Self::row_to_member).collect();
        members
    }

    /// Get active members
    pub async fn get_active(pool: &sqlx::Pool<Any>) -> PoolResult<Vec<PoolMember>> {
        let rows = sqlx::query("SELECT * FROM pool_members WHERE status = 'active'")
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get active members: {}", e)))?;

        let members: Result<Vec<_>, _> = rows.iter().map(Self::row_to_member).collect();
        members
    }

    /// Get members that support a specific model
    pub async fn get_by_model(pool: &sqlx::Pool<Any>, model_id: &str) -> PoolResult<Vec<PoolMember>> {
        // SQLite doesn't have native JSON array contains, so we use LIKE
        let pattern = format!("%\"{}%", model_id);
        
        let rows = sqlx::query("SELECT * FROM pool_members WHERE models LIKE ? AND status = 'active'")
            .bind(&pattern)
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get members by model: {}", e)))?;

        let members: Result<Vec<_>, _> = rows.iter().map(Self::row_to_member).collect();
        members
    }

    /// Update member status
    pub async fn update_status(
        pool: &sqlx::Pool<Any>,
        node_id: &str,
        status: MemberStatus,
    ) -> PoolResult<()> {
        let status_str = status_to_str(&status);

        sqlx::query("UPDATE pool_members SET status = ? WHERE node_id = ?")
            .bind(status_str)
            .bind(node_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update status: {}", e)))?;

        Ok(())
    }

    /// Update heartbeat
    pub async fn update_heartbeat(
        pool: &sqlx::Pool<Any>,
        node_id: &str,
        timestamp: DateTime<Utc>,
    ) -> PoolResult<()> {
        sqlx::query("UPDATE pool_members SET last_heartbeat = ?, status = 'active' WHERE node_id = ?")
            .bind(timestamp.to_rfc3339())
            .bind(node_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update heartbeat: {}", e)))?;

        Ok(())
    }

    /// Update metrics
    pub async fn update_metrics(
        pool: &sqlx::Pool<Any>,
        node_id: &str,
        metrics: &MemberMetrics,
    ) -> PoolResult<()> {
        let metrics_json = serde_json::to_string(metrics)
            .map_err(|e| PoolError::internal(format!("Failed to serialize metrics: {}", e)))?;

        sqlx::query("UPDATE pool_members SET metrics = ? WHERE node_id = ?")
            .bind(&metrics_json)
            .bind(node_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update metrics: {}", e)))?;

        Ok(())
    }

    /// Delete a member
    pub async fn delete(pool: &sqlx::Pool<Any>, node_id: &str) -> PoolResult<bool> {
        let result = sqlx::query("DELETE FROM pool_members WHERE node_id = ?")
            .bind(node_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to delete member: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Count members by status
    pub async fn count_by_status(pool: &sqlx::Pool<Any>) -> PoolResult<(u32, u32)> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active
            FROM pool_members
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to count members: {}", e)))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let active: i64 = row.try_get("active").unwrap_or(0);

        Ok((total as u32, active as u32))
    }

    /// Convert database row to PoolMember
    fn row_to_member(row: &sqlx::any::AnyRow) -> PoolResult<PoolMember> {
        let capabilities_json: String = row.try_get("capabilities").unwrap_or_default();
        let models_json: String = row.try_get("models").unwrap_or_default();
        let multiaddrs_json: String = row.try_get("multiaddrs").unwrap_or_default();
        let metrics_json: String = row.try_get("metrics").unwrap_or_default();
        let status_str: String = row.try_get("status").unwrap_or_default();
        let registered_at_str: String = row.try_get("registered_at").unwrap_or_default();
        let last_heartbeat_str: String = row.try_get("last_heartbeat").unwrap_or_default();

        let capabilities: NodeCapabilities = serde_json::from_str(&capabilities_json)
            .unwrap_or_default();

        let models: Vec<ModelInfo> = serde_json::from_str(&models_json)
            .unwrap_or_default();

        let multiaddrs: Vec<String> = serde_json::from_str(&multiaddrs_json)
            .unwrap_or_default();

        let metrics: MemberMetrics = serde_json::from_str(&metrics_json)
            .unwrap_or_default();

        let status = str_to_status(&status_str);

        let registered_at = DateTime::parse_from_rfc3339(&registered_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let last_heartbeat = DateTime::parse_from_rfc3339(&last_heartbeat_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let node_id: String = row.try_get("node_id").unwrap_or_default();

        Ok(PoolMember {
            member_id: node_id.clone(), // Use node_id as member_id
            node_id,
            peer_id: row.try_get("peer_id").unwrap_or_default(),
            capabilities,
            models,
            status,
            last_heartbeat,
            metrics,
            lightning_pubkey: row.try_get("lightning_pubkey").unwrap_or_default(),
            multiaddrs,
            registered_at,
            channel_id: None,
        })
    }
}

fn status_to_str(status: &MemberStatus) -> &'static str {
    match status {
        MemberStatus::Active => "active",
        MemberStatus::Inactive => "inactive",
        MemberStatus::Draining => "draining",
        MemberStatus::AtRisk => "at_risk",
        MemberStatus::Pending => "pending",
    }
}

fn str_to_status(s: &str) -> MemberStatus {
    match s {
        "active" => MemberStatus::Active,
        "inactive" => MemberStatus::Inactive,
        "draining" => MemberStatus::Draining,
        "at_risk" => MemberStatus::AtRisk,
        "pending" => MemberStatus::Pending,
        _ => MemberStatus::Inactive,
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

    fn create_test_member(node_id: &str) -> PoolMember {
        PoolMember {
            member_id: node_id.to_string(),
            node_id: node_id.to_string(),
            peer_id: format!("peer-{}", node_id),
            capabilities: NodeCapabilities::default(),
            models: vec![ModelInfo {
                model_id: "gpt-4".to_string(),
                family: None,
                parameters: None,
                context_length: Some(8192),
                quantization: None,
            }],
            status: MemberStatus::Active,
            last_heartbeat: Utc::now(),
            metrics: MemberMetrics::default(),
            lightning_pubkey: "02abc123".to_string(),
            multiaddrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            registered_at: Utc::now(),
            channel_id: None,
        }
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let pool = setup_db().await;
        let member = create_test_member("node-1");

        MemberQueries::insert(&pool, &member).await.unwrap();

        let loaded = MemberQueries::get_by_id(&pool, "node-1").await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.node_id, "node-1");
        assert_eq!(loaded.status, MemberStatus::Active);
    }

    #[tokio::test]
    async fn test_get_all() {
        let pool = setup_db().await;

        MemberQueries::insert(&pool, &create_test_member("node-1")).await.unwrap();
        MemberQueries::insert(&pool, &create_test_member("node-2")).await.unwrap();

        let all = MemberQueries::get_all(&pool).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update_status() {
        let pool = setup_db().await;
        MemberQueries::insert(&pool, &create_test_member("node-1")).await.unwrap();

        MemberQueries::update_status(&pool, "node-1", MemberStatus::Draining).await.unwrap();

        let loaded = MemberQueries::get_by_id(&pool, "node-1").await.unwrap().unwrap();
        assert_eq!(loaded.status, MemberStatus::Draining);
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = setup_db().await;
        MemberQueries::insert(&pool, &create_test_member("node-1")).await.unwrap();

        let deleted = MemberQueries::delete(&pool, "node-1").await.unwrap();
        assert!(deleted);

        let loaded = MemberQueries::get_by_id(&pool, "node-1").await.unwrap();
        assert!(loaded.is_none());
    }
}
