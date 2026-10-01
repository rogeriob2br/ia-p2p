//! API key queries.

use chrono::{DateTime, Utc};
use sqlx::{Any, Row};

use crate::auth::{ApiKeyInfo, ClientInfo, RateLimit};
use crate::error::{PoolError, PoolResult};
use crate::models::SlaTier;

/// Queries for API keys
pub struct ApiKeyQueries;

impl ApiKeyQueries {
    /// Insert a new API key
    pub async fn insert(
        pool: &sqlx::Pool<Any>,
        key_id: &str,
        key_hash: &str,
        client_id: &str,
        tier: &SlaTier,
        rate_limit: &RateLimit,
        expires_at: Option<DateTime<Utc>>,
    ) -> PoolResult<()> {
        let tier_str = tier_to_str(tier);

        let rate_limit_json = serde_json::to_string(rate_limit)
            .map_err(|e| PoolError::internal(format!("Failed to serialize rate_limit: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO api_keys (key_id, key_hash, client_id, tier, rate_limit, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(key_id)
        .bind(key_hash)
        .bind(client_id)
        .bind(tier_str)
        .bind(&rate_limit_json)
        .bind(Utc::now().to_rfc3339())
        .bind(expires_at.map(|dt| dt.to_rfc3339()))
        .execute(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to insert API key: {}", e)))?;

        Ok(())
    }

    /// Get API key info by hash
    pub async fn get_by_hash(pool: &sqlx::Pool<Any>, key_hash: &str) -> PoolResult<Option<ApiKeyInfo>> {
        let row = sqlx::query("SELECT * FROM api_keys WHERE key_hash = ? AND revoked_at IS NULL")
            .bind(key_hash)
            .fetch_optional(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get API key: {}", e)))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_key_info(&row)?)),
            None => Ok(None),
        }
    }

    /// Get API key info by key_id
    pub async fn get_by_id(pool: &sqlx::Pool<Any>, key_id: &str) -> PoolResult<Option<ApiKeyInfo>> {
        let row = sqlx::query("SELECT * FROM api_keys WHERE key_id = ?")
            .bind(key_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get API key: {}", e)))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_key_info(&row)?)),
            None => Ok(None),
        }
    }

    /// Get all API keys (for admin listing)
    pub async fn get_all(pool: &sqlx::Pool<Any>) -> PoolResult<Vec<ApiKeyInfo>> {
        let rows = sqlx::query("SELECT * FROM api_keys ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get API keys: {}", e)))?;

        let keys: Result<Vec<_>, _> = rows.iter().map(Self::row_to_key_info).collect();
        keys
    }

    /// Revoke an API key
    pub async fn revoke(pool: &sqlx::Pool<Any>, key_id: &str) -> PoolResult<bool> {
        let result = sqlx::query("UPDATE api_keys SET revoked_at = ? WHERE key_id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(key_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to revoke API key: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Update last used timestamp
    pub async fn update_last_used(pool: &sqlx::Pool<Any>, key_id: &str) -> PoolResult<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = ?, usage_count = usage_count + 1 WHERE key_id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(key_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update last_used: {}", e)))?;

        Ok(())
    }

    /// Check if key is valid (not expired, not revoked)
    pub async fn is_valid(pool: &sqlx::Pool<Any>, key_hash: &str) -> PoolResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1 FROM api_keys 
            WHERE key_hash = ? 
            AND revoked_at IS NULL 
            AND (expires_at IS NULL OR expires_at > ?)
            "#,
        )
        .bind(key_hash)
        .bind(Utc::now().to_rfc3339())
        .fetch_optional(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to check API key: {}", e)))?;

        Ok(row.is_some())
    }

    /// Get client info from key hash
    pub async fn get_client_info(pool: &sqlx::Pool<Any>, key_hash: &str) -> PoolResult<Option<ClientInfo>> {
        let row = sqlx::query(
            "SELECT client_id, tier, rate_limit FROM api_keys WHERE key_hash = ? AND revoked_at IS NULL"
        )
        .bind(key_hash)
        .fetch_optional(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to get client info: {}", e)))?;

        match row {
            Some(row) => {
                let tier_str: String = row.try_get("tier").unwrap_or_default();
                let rate_limit_json: String = row.try_get("rate_limit").unwrap_or_default();

                let tier = str_to_tier(&tier_str);
                let rate_limit: RateLimit = serde_json::from_str(&rate_limit_json)
                    .unwrap_or_default();

                Ok(Some(ClientInfo {
                    key_id: String::new(), // Not needed for this query
                    client_id: row.try_get("client_id").unwrap_or_default(),
                    client_name: String::new(),
                    tier,
                }))
            }
            None => Ok(None),
        }
    }

    /// Convert row to ApiKeyInfo
    fn row_to_key_info(row: &sqlx::any::AnyRow) -> PoolResult<ApiKeyInfo> {
        let tier_str: String = row.try_get("tier").unwrap_or_default();
        let rate_limit_json: String = row.try_get("rate_limit").unwrap_or_default();
        let created_at_str: String = row.try_get("created_at").unwrap_or_default();
        let expires_at_str: Option<String> = row.try_get("expires_at").ok();
        let revoked_at_str: Option<String> = row.try_get("revoked_at").ok();
        let last_used_at_str: Option<String> = row.try_get("last_used_at").ok();

        let tier = str_to_tier(&tier_str);
        let rate_limit: RateLimit = serde_json::from_str(&rate_limit_json)
            .unwrap_or_default();

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let expires_at = expires_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        let revoked_at = revoked_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        let last_used_at = last_used_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Ok(ApiKeyInfo {
            key_id: row.try_get("key_id").unwrap_or_default(),
            key_hash: row.try_get("key_hash").unwrap_or_default(),
            client_id: row.try_get("client_id").unwrap_or_default(),
            client_name: String::new(),
            tier,
            rate_limit,
            created_at,
            expires_at,
            revoked_at,
            is_active: revoked_at.is_none(),
            last_used_at,
            usage_count: row.try_get::<i64, _>("usage_count").unwrap_or(0) as u64,
        })
    }
}

fn tier_to_str(tier: &SlaTier) -> &'static str {
    match tier {
        SlaTier::Free => "free",
        SlaTier::Basic => "basic",
        SlaTier::Pro => "pro",
        SlaTier::Enterprise => "enterprise",
    }
}

fn str_to_tier(s: &str) -> SlaTier {
    match s {
        "free" => SlaTier::Free,
        "basic" => SlaTier::Basic,
        "pro" => SlaTier::Pro,
        "enterprise" => SlaTier::Enterprise,
        _ => SlaTier::Free,
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

    #[tokio::test]
    async fn test_insert_and_get() {
        let pool = setup_db().await;

        let rate_limit = RateLimit {
            requests_per_minute: 60,
            requests_per_day: 1000,
            tokens_per_minute: 10000,
            tokens_per_day: 100000,
            concurrent_requests: 10,
        };

        ApiKeyQueries::insert(
            &pool,
            "key-1",
            "hash-abc123",
            "client-1",
            &SlaTier::Basic,
            &rate_limit,
            None,
        )
        .await
        .unwrap();

        let key = ApiKeyQueries::get_by_hash(&pool, "hash-abc123").await.unwrap();
        assert!(key.is_some());

        let key = key.unwrap();
        assert_eq!(key.key_id, "key-1");
        assert_eq!(key.client_id, "client-1");
    }

    #[tokio::test]
    async fn test_revoke() {
        let pool = setup_db().await;

        ApiKeyQueries::insert(
            &pool,
            "key-1",
            "hash-abc123",
            "client-1",
            &SlaTier::Basic,
            &RateLimit::default(),
            None,
        )
        .await
        .unwrap();

        // Revoke
        let revoked = ApiKeyQueries::revoke(&pool, "key-1").await.unwrap();
        assert!(revoked);

        // Should not find revoked key
        let key = ApiKeyQueries::get_by_hash(&pool, "hash-abc123").await.unwrap();
        assert!(key.is_none());
    }

    #[tokio::test]
    async fn test_is_valid() {
        let pool = setup_db().await;

        ApiKeyQueries::insert(
            &pool,
            "key-1",
            "hash-abc123",
            "client-1",
            &SlaTier::Basic,
            &RateLimit::default(),
            None,
        )
        .await
        .unwrap();

        assert!(ApiKeyQueries::is_valid(&pool, "hash-abc123").await.unwrap());

        // Revoke and check
        ApiKeyQueries::revoke(&pool, "key-1").await.unwrap();
        assert!(!ApiKeyQueries::is_valid(&pool, "hash-abc123").await.unwrap());
    }
}
