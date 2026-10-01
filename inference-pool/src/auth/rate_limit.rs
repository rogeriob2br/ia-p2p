//! Rate limiting implementation.

use super::RateLimitConfig;
use crate::error::{PoolError, PoolResult};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::collections::VecDeque;

/// Rate limiter state for a client
#[derive(Debug, Clone)]
struct ClientRateState {
    /// Recent request timestamps (for RPM calculation)
    request_times: VecDeque<DateTime<Utc>>,

    /// Tokens used today
    tokens_today: u64,

    /// Day start timestamp
    day_start: DateTime<Utc>,

    /// Current concurrent requests
    concurrent: u32,

    /// Configuration
    config: RateLimitConfig,
}

impl ClientRateState {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            request_times: VecDeque::new(),
            tokens_today: 0,
            day_start: Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            concurrent: 0,
            config,
        }
    }

    /// Clean up old request times
    fn cleanup_old_requests(&mut self) {
        let minute_ago = Utc::now() - Duration::minutes(1);
        while let Some(front) = self.request_times.front() {
            if *front < minute_ago {
                self.request_times.pop_front();
            } else {
                break;
            }
        }
    }

    /// Reset daily counter if new day
    fn check_daily_reset(&mut self) {
        let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        if today > self.day_start {
            self.tokens_today = 0;
            self.day_start = today;
        }
    }

    /// Check if request is allowed
    fn check_request(&mut self) -> PoolResult<()> {
        self.cleanup_old_requests();
        self.check_daily_reset();

        // Check RPM
        if self.request_times.len() >= self.config.requests_per_minute as usize {
            let oldest = self.request_times.front().unwrap();
            let wait_time = (*oldest + Duration::minutes(1)) - Utc::now();
            return Err(PoolError::rate_limit(format!(
                "Rate limit exceeded. Try again in {} seconds.",
                wait_time.num_seconds().max(1)
            )));
        }

        // Check concurrent
        if self.concurrent >= self.config.max_concurrent {
            return Err(PoolError::rate_limit(
                "Too many concurrent requests. Please wait for some to complete.",
            ));
        }

        Ok(())
    }

    /// Check if tokens allowed
    fn check_tokens(&self, tokens: u64) -> PoolResult<()> {
        if self.tokens_today + tokens > self.config.tokens_per_day {
            return Err(PoolError::rate_limit(format!(
                "Daily token limit exceeded. Used: {}, Limit: {}",
                self.tokens_today, self.config.tokens_per_day
            )));
        }
        Ok(())
    }

    /// Record a new request
    fn record_request(&mut self) {
        self.request_times.push_back(Utc::now());
        self.concurrent += 1;
    }

    /// Complete a request
    fn complete_request(&mut self, tokens: u64) {
        self.concurrent = self.concurrent.saturating_sub(1);
        self.tokens_today += tokens;
    }
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for a client
    async fn check_request(&self, client_id: &str, config: &RateLimitConfig) -> PoolResult<()>;

    /// Record the start of a request
    async fn start_request(&self, client_id: &str, config: &RateLimitConfig) -> PoolResult<()>;

    /// Record the completion of a request
    async fn complete_request(&self, client_id: &str, tokens: u64) -> PoolResult<()>;

    /// Check if tokens are available
    async fn check_tokens(
        &self,
        client_id: &str,
        tokens: u64,
        config: &RateLimitConfig,
    ) -> PoolResult<()>;

    /// Get current usage for a client
    fn get_usage(&self, client_id: &str) -> Option<ClientUsage>;
}

/// Client usage statistics
#[derive(Debug, Clone)]
pub struct ClientUsage {
    /// Requests in the last minute
    pub requests_last_minute: u32,

    /// Tokens used today
    pub tokens_today: u64,

    /// Current concurrent requests
    pub concurrent_requests: u32,
}

/// In-memory rate limiter implementation
#[derive(Debug)]
pub struct RateLimiterImpl {
    /// Client states
    states: DashMap<String, ClientRateState>,
}

impl RateLimiterImpl {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }
}

impl Default for RateLimiterImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimiter for RateLimiterImpl {
    async fn check_request(&self, client_id: &str, config: &RateLimitConfig) -> PoolResult<()> {
        let mut state = self
            .states
            .entry(client_id.to_string())
            .or_insert_with(|| ClientRateState::new(config.clone()));

        state.check_request()
    }

    async fn start_request(&self, client_id: &str, config: &RateLimitConfig) -> PoolResult<()> {
        let mut state = self
            .states
            .entry(client_id.to_string())
            .or_insert_with(|| ClientRateState::new(config.clone()));

        state.check_request()?;
        state.record_request();
        Ok(())
    }

    async fn complete_request(&self, client_id: &str, tokens: u64) -> PoolResult<()> {
        if let Some(mut state) = self.states.get_mut(client_id) {
            state.complete_request(tokens);
        }
        Ok(())
    }

    async fn check_tokens(
        &self,
        client_id: &str,
        tokens: u64,
        config: &RateLimitConfig,
    ) -> PoolResult<()> {
        let mut state = self
            .states
            .entry(client_id.to_string())
            .or_insert_with(|| ClientRateState::new(config.clone()));

        state.check_daily_reset();
        state.check_tokens(tokens)
    }

    fn get_usage(&self, client_id: &str) -> Option<ClientUsage> {
        self.states.get(client_id).map(|state| ClientUsage {
            requests_last_minute: state.request_times.len() as u32,
            tokens_today: state.tokens_today,
            concurrent_requests: state.concurrent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_check() {
        let limiter = RateLimiterImpl::new();
        let config = RateLimitConfig {
            requests_per_minute: 5,
            tokens_per_day: 1000,
            max_concurrent: 2,
        };

        // Should allow first requests
        for _ in 0..5 {
            limiter.start_request("client-1", &config).await.unwrap();
            limiter.complete_request("client-1", 0).await.unwrap();
        }

        // Should block next request (RPM exceeded)
        let result = limiter.check_request("client-1", &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        let limiter = RateLimiterImpl::new();
        let config = RateLimitConfig {
            requests_per_minute: 100,
            tokens_per_day: 1000,
            max_concurrent: 2,
        };

        // Start 2 requests (max concurrent)
        limiter.start_request("client-1", &config).await.unwrap();
        limiter.start_request("client-1", &config).await.unwrap();

        // Third should fail
        let result = limiter.start_request("client-1", &config).await;
        assert!(result.is_err());

        // Complete one
        limiter.complete_request("client-1", 0).await.unwrap();

        // Now should succeed
        limiter.start_request("client-1", &config).await.unwrap();
    }

    #[tokio::test]
    async fn test_token_limit() {
        let limiter = RateLimiterImpl::new();
        let config = RateLimitConfig {
            requests_per_minute: 100,
            tokens_per_day: 100,
            max_concurrent: 10,
        };

        // Use some tokens
        limiter.start_request("client-1", &config).await.unwrap();
        limiter.complete_request("client-1", 50).await.unwrap();

        // Check tokens
        limiter.check_tokens("client-1", 40, &config).await.unwrap();

        // Try to use more than available
        let result = limiter.check_tokens("client-1", 60, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_usage() {
        let limiter = RateLimiterImpl::new();
        let config = RateLimitConfig::default();

        limiter.start_request("client-1", &config).await.unwrap();
        limiter.complete_request("client-1", 100).await.unwrap();

        let usage = limiter.get_usage("client-1").unwrap();
        assert_eq!(usage.requests_last_minute, 1);
        assert_eq!(usage.tokens_today, 100);
        assert_eq!(usage.concurrent_requests, 0);
    }
}







