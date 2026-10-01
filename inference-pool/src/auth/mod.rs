//! Authentication module.
//!
//! Validates API Keys and manages rate limiting.
//!
//! ## Features
//!
//! - API key validation
//! - Client association
//! - Rate limiting per client
//! - Usage tracking
//! - SLA tier management

mod middleware;
mod rate_limit;
mod storage;

pub use middleware::{auth_middleware, AuthState, ClientInfo, extract_api_key};
pub use rate_limit::{RateLimiter, RateLimiterImpl};
pub use storage::InMemoryAuthStorage;

use crate::error::PoolResult;
use crate::models::SlaTier;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// API Key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    /// Unique key identifier
    pub key_id: String,

    /// The actual API key (hashed in storage)
    #[serde(skip_serializing)]
    #[serde(default)]
    pub key_hash: String,

    /// Associated client ID
    pub client_id: String,

    /// Client name (optional for backward compatibility)
    #[serde(default)]
    pub client_name: String,

    /// SLA tier
    pub tier: SlaTier,

    /// Rate limit configuration
    pub rate_limit: RateLimit,

    /// When the key was created
    pub created_at: DateTime<Utc>,

    /// When the key expires (None = never)
    pub expires_at: Option<DateTime<Utc>>,

    /// When the key was revoked (None = not revoked)
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,

    /// Whether the key is active
    #[serde(default = "default_is_active")]
    pub is_active: bool,

    /// Last used timestamp
    #[serde(default)]
    pub last_used_at: Option<DateTime<Utc>>,

    /// Total requests made
    #[serde(default)]
    pub usage_count: u64,
}

fn default_is_active() -> bool {
    true
}

impl ApiKeyInfo {
    /// Check if key is valid (not expired, not revoked)
    pub fn is_valid(&self) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        if let Some(expires) = self.expires_at {
            if expires < Utc::now() {
                return false;
            }
        }
        self.is_active
    }
}

/// Rate limit configuration (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,

    /// Tokens per day
    pub tokens_per_day: u64,

    /// Maximum concurrent requests
    pub max_concurrent: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_day: 1_000_000,
            max_concurrent: 10,
        }
    }
}

/// Rate limit settings (for database persistence)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimit {
    /// Requests per minute
    pub requests_per_minute: u32,

    /// Requests per day
    pub requests_per_day: u32,

    /// Tokens per minute
    pub tokens_per_minute: u32,

    /// Tokens per day
    pub tokens_per_day: u64,

    /// Concurrent requests
    pub concurrent_requests: u32,
}

impl From<RateLimitConfig> for RateLimit {
    fn from(config: RateLimitConfig) -> Self {
        Self {
            requests_per_minute: config.requests_per_minute,
            requests_per_day: config.requests_per_minute * 60 * 24,
            tokens_per_minute: (config.tokens_per_day / 1440) as u32,
            tokens_per_day: config.tokens_per_day,
            concurrent_requests: config.max_concurrent,
        }
    }
}

impl From<RateLimit> for RateLimitConfig {
    fn from(rate: RateLimit) -> Self {
        Self {
            requests_per_minute: rate.requests_per_minute,
            tokens_per_day: rate.tokens_per_day,
            max_concurrent: rate.concurrent_requests,
        }
    }
}

impl From<&RateLimit> for RateLimitConfig {
    fn from(rate: &RateLimit) -> Self {
        Self {
            requests_per_minute: rate.requests_per_minute,
            tokens_per_day: rate.tokens_per_day,
            max_concurrent: rate.concurrent_requests,
        }
    }
}

impl RateLimitConfig {
    /// Create rate limit for a specific SLA tier
    pub fn for_tier(tier: &SlaTier) -> Self {
        match tier {
            SlaTier::Free => Self {
                requests_per_minute: 10,
                tokens_per_day: 10_000,
                max_concurrent: 2,
            },
            SlaTier::Basic => Self {
                requests_per_minute: 60,
                tokens_per_day: 100_000,
                max_concurrent: 5,
            },
            SlaTier::Pro => Self {
                requests_per_minute: 300,
                tokens_per_day: 1_000_000,
                max_concurrent: 20,
            },
            SlaTier::Enterprise => Self {
                requests_per_minute: 1000,
                tokens_per_day: 10_000_000,
                max_concurrent: 100,
            },
        }
    }
}

/// Auth Storage trait
#[async_trait]
pub trait AuthStorage: Send + Sync {
    /// Validate an API key and return its info
    async fn validate_key(&self, api_key: &str) -> PoolResult<ApiKeyInfo>;

    /// Get key info by key_id
    async fn get_key_info(&self, key_id: &str) -> PoolResult<Option<ApiKeyInfo>>;

    /// Create a new API key
    async fn create_key(
        &self,
        client_id: &str,
        client_name: &str,
        tier: SlaTier,
    ) -> PoolResult<(String, ApiKeyInfo)>; // Returns (api_key, info)

    /// Revoke an API key
    async fn revoke_key(&self, key_id: &str) -> PoolResult<()>;

    /// Update key usage statistics
    async fn record_usage(&self, key_id: &str, tokens: u64) -> PoolResult<()>;

    /// List all keys for a client
    async fn list_client_keys(&self, client_id: &str) -> PoolResult<Vec<ApiKeyInfo>>;

    /// Update key tier
    async fn update_tier(&self, key_id: &str, tier: SlaTier) -> PoolResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_for_tier() {
        let free = RateLimitConfig::for_tier(&SlaTier::Free);
        let enterprise = RateLimitConfig::for_tier(&SlaTier::Enterprise);

        assert!(free.requests_per_minute < enterprise.requests_per_minute);
        assert!(free.tokens_per_day < enterprise.tokens_per_day);
    }
}
