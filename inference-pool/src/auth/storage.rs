//! In-memory storage for authentication data.

use super::{ApiKeyInfo, AuthStorage, RateLimit, RateLimitConfig};
use crate::error::{PoolError, PoolResult};
use crate::models::SlaTier;
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use sha2::{Digest, Sha256};

/// In-memory authentication storage
#[derive(Debug)]
pub struct InMemoryAuthStorage {
    /// Keys indexed by hash
    keys_by_hash: DashMap<String, ApiKeyInfo>,

    /// Keys indexed by key_id
    keys_by_id: DashMap<String, String>, // key_id -> key_hash

    /// Client keys index
    client_keys: DashMap<String, Vec<String>>, // client_id -> [key_ids]

    /// API key prefix
    key_prefix: String,
}

impl InMemoryAuthStorage {
    /// Create a new in-memory auth storage
    pub fn new() -> Self {
        Self {
            keys_by_hash: DashMap::new(),
            keys_by_id: DashMap::new(),
            client_keys: DashMap::new(),
            key_prefix: "sk-pool-".to_string(),
        }
    }

    /// Create with custom key prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            keys_by_hash: DashMap::new(),
            keys_by_id: DashMap::new(),
            client_keys: DashMap::new(),
            key_prefix: prefix.into(),
        }
    }

    /// Hash an API key
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Generate a new API key
    fn generate_key(&self) -> String {
        let random_part: String = (0..32)
            .map(|_| {
                let idx = rand::random::<usize>() % 62;
                let chars: Vec<char> =
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                        .chars()
                        .collect();
                chars[idx]
            })
            .collect();

        format!("{}{}", self.key_prefix, random_part)
    }

    /// Add a pre-created key (for testing)
    pub fn add_key(&self, api_key: &str, info: ApiKeyInfo) {
        let hash = Self::hash_key(api_key);
        let key_id = info.key_id.clone();
        let client_id = info.client_id.clone();

        self.keys_by_hash.insert(hash.clone(), info);
        self.keys_by_id.insert(key_id.clone(), hash);
        self.client_keys
            .entry(client_id)
            .or_insert_with(Vec::new)
            .push(key_id);
    }
}

impl Default for InMemoryAuthStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthStorage for InMemoryAuthStorage {
    async fn validate_key(&self, api_key: &str) -> PoolResult<ApiKeyInfo> {
        // Check prefix
        if !api_key.starts_with(&self.key_prefix) {
            return Err(PoolError::api_key_invalid());
        }

        let hash = Self::hash_key(api_key);

        let mut info = self
            .keys_by_hash
            .get_mut(&hash)
            .ok_or_else(PoolError::api_key_invalid)?;

        // Check if active
        if !info.is_active {
            return Err(PoolError::api_key_invalid());
        }

        // Check expiration
        if let Some(expires) = info.expires_at {
            if Utc::now() > expires {
                return Err(PoolError::api_key_expired());
            }
        }

        // Update last used
        info.last_used_at = Some(Utc::now());
        info.usage_count += 1;

        Ok(info.clone())
    }

    async fn get_key_info(&self, key_id: &str) -> PoolResult<Option<ApiKeyInfo>> {
        let hash = match self.keys_by_id.get(key_id) {
            Some(h) => h.clone(),
            None => return Ok(None),
        };

        Ok(self.keys_by_hash.get(&hash).map(|info| info.clone()))
    }

    async fn create_key(
        &self,
        client_id: &str,
        client_name: &str,
        tier: SlaTier,
    ) -> PoolResult<(String, ApiKeyInfo)> {
        let api_key = self.generate_key();
        let hash = Self::hash_key(&api_key);
        let key_id = format!("key-{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let info = ApiKeyInfo {
            key_id: key_id.clone(),
            key_hash: hash.clone(),
            client_id: client_id.to_string(),
            client_name: client_name.to_string(),
            tier: tier.clone(),
            rate_limit: RateLimitConfig::for_tier(&tier).into(),
            created_at: now,
            expires_at: None,
            revoked_at: None,
            is_active: true,
            last_used_at: None,
            usage_count: 0,
        };

        self.keys_by_hash.insert(hash.clone(), info.clone());
        self.keys_by_id.insert(key_id.clone(), hash);
        self.client_keys
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(key_id);

        tracing::info!(
            client_id = %client_id,
            tier = ?tier,
            "Created new API key"
        );

        Ok((api_key, info))
    }

    async fn revoke_key(&self, key_id: &str) -> PoolResult<()> {
        let hash = self
            .keys_by_id
            .get(key_id)
            .ok_or_else(|| PoolError::invalid_request(format!("Key {} not found", key_id)))?
            .clone();

        if let Some(mut info) = self.keys_by_hash.get_mut(&hash) {
            info.is_active = false;
        }

        tracing::info!(key_id = %key_id, "Revoked API key");

        Ok(())
    }

    async fn record_usage(&self, key_id: &str, _tokens: u64) -> PoolResult<()> {
        let hash = self
            .keys_by_id
            .get(key_id)
            .ok_or_else(|| PoolError::invalid_request(format!("Key {} not found", key_id)))?
            .clone();

        if let Some(mut info) = self.keys_by_hash.get_mut(&hash) {
            info.usage_count += 1;
        }

        Ok(())
    }

    async fn list_client_keys(&self, client_id: &str) -> PoolResult<Vec<ApiKeyInfo>> {
        let key_ids = match self.client_keys.get(client_id) {
            Some(ids) => ids.clone(),
            None => return Ok(vec![]),
        };

        let mut keys = Vec::new();
        for key_id in key_ids {
            if let Ok(Some(info)) = self.get_key_info(&key_id).await {
                keys.push(info);
            }
        }

        Ok(keys)
    }

    async fn update_tier(&self, key_id: &str, tier: SlaTier) -> PoolResult<()> {
        let hash = self
            .keys_by_id
            .get(key_id)
            .ok_or_else(|| PoolError::invalid_request(format!("Key {} not found", key_id)))?
            .clone();

        if let Some(mut info) = self.keys_by_hash.get_mut(&hash) {
            info.tier = tier.clone();
            info.rate_limit = RateLimitConfig::for_tier(&tier).into();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_validate_key() {
        let storage = InMemoryAuthStorage::new();

        let (api_key, info) = storage
            .create_key("client-1", "Test Client", SlaTier::Basic)
            .await
            .unwrap();

        assert!(api_key.starts_with("sk-pool-"));
        assert_eq!(info.client_id, "client-1");
        assert_eq!(info.tier, SlaTier::Basic);

        // Validate the key
        let validated = storage.validate_key(&api_key).await.unwrap();
        assert_eq!(validated.key_id, info.key_id);
        assert_eq!(validated.usage_count, 1); // Incremented on validate
    }

    #[tokio::test]
    async fn test_invalid_key_fails() {
        let storage = InMemoryAuthStorage::new();

        let result = storage.validate_key("sk-pool-invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wrong_prefix_fails() {
        let storage = InMemoryAuthStorage::new();

        let result = storage.validate_key("wrong-prefix-key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let storage = InMemoryAuthStorage::new();

        let (api_key, info) = storage
            .create_key("client-1", "Test Client", SlaTier::Basic)
            .await
            .unwrap();

        // Revoke
        storage.revoke_key(&info.key_id).await.unwrap();

        // Should fail validation
        let result = storage.validate_key(&api_key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_usage() {
        let storage = InMemoryAuthStorage::new();

        let (_, info) = storage
            .create_key("client-1", "Test Client", SlaTier::Basic)
            .await
            .unwrap();

        storage.record_usage(&info.key_id, 1000).await.unwrap();

        let updated = storage.get_key_info(&info.key_id).await.unwrap().unwrap();
        assert_eq!(updated.usage_count, 1); // usage_count incremented
    }

    #[tokio::test]
    async fn test_list_client_keys() {
        let storage = InMemoryAuthStorage::new();

        // Create multiple keys for same client
        storage
            .create_key("client-1", "Test Client", SlaTier::Basic)
            .await
            .unwrap();
        storage
            .create_key("client-1", "Test Client", SlaTier::Pro)
            .await
            .unwrap();

        let keys = storage.list_client_keys("client-1").await.unwrap();
        assert_eq!(keys.len(), 2);
    }
}






