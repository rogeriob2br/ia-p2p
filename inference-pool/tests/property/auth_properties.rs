//! Property tests for Authentication and Rate Limiting.
//!
//! Feature: auth
//! Tests Properties 13, 14 from the design document.

use chrono::{Duration, Utc};
use inference_pool::auth::{
    ApiKeyInfo, AuthStorage, InMemoryAuthStorage, RateLimitConfig, RateLimiter, RateLimiterImpl,
};
use inference_pool::models::SlaTier;
use proptest::prelude::*;
use std::sync::Arc;

/// Strategy for generating client IDs
fn client_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9]{8}")
        .unwrap()
        .prop_map(|s| format!("client-{}", s))
}

/// Strategy for generating SLA tiers
fn tier_strategy() -> impl Strategy<Value = SlaTier> {
    prop_oneof![
        Just(SlaTier::Free),
        Just(SlaTier::Basic),
        Just(SlaTier::Pro),
        Just(SlaTier::Enterprise),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: auth
    /// Property 13: Valid API Key Grants Access
    ///
    /// For any valid API key, the authentication middleware SHALL
    /// grant access and return the associated client information.
    ///
    /// Validates: Requirement 4.1
    #[test]
    fn property_13_valid_key_grants_access(
        client_id in client_id_strategy(),
        client_name in prop::string::string_regex("[A-Za-z ]{5,30}").unwrap(),
        tier in tier_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let storage = InMemoryAuthStorage::new();

            // Create a key
            let (api_key, info) = storage
                .create_key(&client_id, &client_name, tier.clone())
                .await
                .unwrap();

            prop_assert!(api_key.starts_with("sk-pool-"), "Key should have correct prefix");

            // Validate the key
            let validated = storage.validate_key(&api_key).await.unwrap();

            prop_assert_eq!(&validated.client_id, &client_id);
            prop_assert_eq!(&validated.client_name, &client_name);
            prop_assert_eq!(&validated.tier, &tier);
            prop_assert!(validated.is_active);
            prop_assert!(validated.usage_count >= 1); // Incremented on validate

            Ok(())
        })?;
    }

    /// Feature: auth
    /// Property 14: Invalid API Key Denies Access
    ///
    /// For any invalid API key, the authentication middleware SHALL
    /// deny access with an appropriate error.
    ///
    /// Validates: Requirement 4.2
    #[test]
    fn property_14_invalid_key_denies_access(
        random_key in prop::string::string_regex("[a-zA-Z0-9]{32}").unwrap(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let storage = InMemoryAuthStorage::new();

            // Try to validate with various invalid keys
            let invalid_keys = vec![
                format!("sk-pool-{}", random_key),  // Wrong key
                format!("sk-{}", random_key),        // Wrong prefix
                random_key.clone(),                   // No prefix
                "".to_string(),                       // Empty
            ];

            for key in invalid_keys {
                let result = storage.validate_key(&key).await;
                prop_assert!(
                    result.is_err(),
                    "Key '{}' should be rejected",
                    key
                );
            }

            Ok(())
        })?;
    }

    /// Feature: auth
    /// Property: Revoked Keys Are Rejected
    ///
    /// For any revoked API key, the authentication middleware SHALL
    /// deny access.
    #[test]
    fn revoked_keys_rejected(
        client_id in client_id_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let storage = InMemoryAuthStorage::new();

            // Create and then revoke a key
            let (api_key, info) = storage
                .create_key(&client_id, "Test Client", SlaTier::Basic)
                .await
                .unwrap();

            // Verify it works
            storage.validate_key(&api_key).await.unwrap();

            // Revoke it
            storage.revoke_key(&info.key_id).await.unwrap();

            // Should now be rejected
            let result = storage.validate_key(&api_key).await;
            prop_assert!(result.is_err(), "Revoked key should be rejected");

            Ok(())
        })?;
    }

    /// Feature: auth
    /// Property: Rate Limits Per Tier
    ///
    /// Each SLA tier SHALL have appropriate rate limits.
    #[test]
    fn rate_limits_increase_with_tier(_dummy in 0..1u32) {
        let free = RateLimitConfig::for_tier(&SlaTier::Free);
        let basic = RateLimitConfig::for_tier(&SlaTier::Basic);
        let pro = RateLimitConfig::for_tier(&SlaTier::Pro);
        let enterprise = RateLimitConfig::for_tier(&SlaTier::Enterprise);

        // Rate limits should increase with tier
        prop_assert!(
            free.requests_per_minute <= basic.requests_per_minute,
            "Basic should have >= Free RPM"
        );
        prop_assert!(
            basic.requests_per_minute <= pro.requests_per_minute,
            "Pro should have >= Basic RPM"
        );
        prop_assert!(
            pro.requests_per_minute <= enterprise.requests_per_minute,
            "Enterprise should have >= Pro RPM"
        );

        // Token limits should increase
        prop_assert!(
            free.tokens_per_day <= basic.tokens_per_day,
            "Basic should have >= Free tokens"
        );
        prop_assert!(
            basic.tokens_per_day <= pro.tokens_per_day,
            "Pro should have >= Basic tokens"
        );
        prop_assert!(
            pro.tokens_per_day <= enterprise.tokens_per_day,
            "Enterprise should have >= Pro tokens"
        );
    }

    /// Feature: rate-limiting
    /// Property: Rate Limiter Enforces RPM
    ///
    /// The rate limiter SHALL reject requests that exceed the RPM limit.
    #[test]
    fn rate_limiter_enforces_rpm(
        rpm_limit in 3u32..10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let limiter = RateLimiterImpl::new();
            let config = RateLimitConfig {
                requests_per_minute: rpm_limit,
                tokens_per_day: 1_000_000,
                max_concurrent: 100,
            };

            // Make RPM requests - should all succeed
            for _ in 0..rpm_limit {
                limiter.start_request("client-1", &config).await.unwrap();
                limiter.complete_request("client-1", 0).await.unwrap();
            }

            // Next request should fail
            let result = limiter.check_request("client-1", &config).await;
            prop_assert!(
                result.is_err(),
                "Should reject after {} requests",
                rpm_limit
            );

            Ok(())
        })?;
    }

    /// Feature: rate-limiting
    /// Property: Rate Limiter Enforces Concurrent Limit
    ///
    /// The rate limiter SHALL reject requests that exceed the concurrent limit.
    #[test]
    fn rate_limiter_enforces_concurrent(
        concurrent_limit in 2u32..10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let limiter = RateLimiterImpl::new();
            let config = RateLimitConfig {
                requests_per_minute: 1000, // High RPM
                tokens_per_day: 1_000_000,
                max_concurrent: concurrent_limit,
            };

            // Start concurrent_limit requests without completing
            for _ in 0..concurrent_limit {
                limiter.start_request("client-1", &config).await.unwrap();
            }

            // Next should fail
            let result = limiter.start_request("client-1", &config).await;
            prop_assert!(
                result.is_err(),
                "Should reject at {} concurrent",
                concurrent_limit
            );

            // Complete one
            limiter.complete_request("client-1", 0).await.unwrap();

            // Should now succeed
            limiter.start_request("client-1", &config).await.unwrap();

            Ok(())
        })?;
    }

    /// Feature: rate-limiting
    /// Property: Token Limit Enforcement
    ///
    /// The rate limiter SHALL reject requests that would exceed the daily token limit.
    #[test]
    fn rate_limiter_enforces_token_limit(
        daily_limit in 100u64..1000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let limiter = RateLimiterImpl::new();
            let config = RateLimitConfig {
                requests_per_minute: 1000,
                tokens_per_day: daily_limit,
                max_concurrent: 100,
            };

            // Use 80% of tokens
            let used = (daily_limit as f64 * 0.8) as u64;
            limiter.start_request("client-1", &config).await.unwrap();
            limiter.complete_request("client-1", used).await.unwrap();

            // Check for remaining 10% - should succeed
            let remaining_ok = (daily_limit as f64 * 0.1) as u64;
            let result = limiter.check_tokens("client-1", remaining_ok, &config).await;
            prop_assert!(result.is_ok(), "Should allow tokens within limit");

            // Check for 30% more - should fail
            let too_much = (daily_limit as f64 * 0.3) as u64;
            let result = limiter.check_tokens("client-1", too_much, &config).await;
            prop_assert!(result.is_err(), "Should reject tokens exceeding limit");

            Ok(())
        })?;
    }

    /// Feature: auth
    /// Property: Usage Tracking Accuracy
    ///
    /// The auth storage SHALL accurately track usage statistics.
    #[test]
    fn usage_tracking_accuracy(
        request_count in 1u64..20,
        tokens_per_request in 10u64..1000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let storage = InMemoryAuthStorage::new();

            let (api_key, info) = storage
                .create_key("client-1", "Test", SlaTier::Basic)
                .await
                .unwrap();

            // Simulate multiple requests with token usage
            // validate_key increments usage_count, and record_usage also increments
            for _ in 0..request_count {
                storage.validate_key(&api_key).await.unwrap();
                storage.record_usage(&info.key_id, tokens_per_request).await.unwrap();
            }

            // Check totals: validate_key and record_usage both increment usage_count
            let updated = storage.get_key_info(&info.key_id).await.unwrap().unwrap();

            prop_assert_eq!(
                updated.usage_count, request_count * 2, // Each loop iteration does 2 increments
                "Usage count should be {}",
                request_count * 2
            );

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_client_keys() {
        let storage = InMemoryAuthStorage::new();

        storage
            .create_key("client-1", "Test", SlaTier::Free)
            .await
            .unwrap();
        storage
            .create_key("client-1", "Test", SlaTier::Basic)
            .await
            .unwrap();
        storage
            .create_key("client-2", "Other", SlaTier::Pro)
            .await
            .unwrap();

        let client1_keys = storage.list_client_keys("client-1").await.unwrap();
        let client2_keys = storage.list_client_keys("client-2").await.unwrap();

        assert_eq!(client1_keys.len(), 2);
        assert_eq!(client2_keys.len(), 1);
    }

    #[tokio::test]
    async fn test_update_tier() {
        let storage = InMemoryAuthStorage::new();

        let (_, info) = storage
            .create_key("client-1", "Test", SlaTier::Free)
            .await
            .unwrap();

        assert_eq!(info.tier, SlaTier::Free);

        storage.update_tier(&info.key_id, SlaTier::Pro).await.unwrap();

        let updated = storage.get_key_info(&info.key_id).await.unwrap().unwrap();
        assert_eq!(updated.tier, SlaTier::Pro);

        // Rate limits should also be updated
        let pro_limits = RateLimitConfig::for_tier(&SlaTier::Pro);
        assert_eq!(
            updated.rate_limit.requests_per_minute,
            pro_limits.requests_per_minute
        );
    }
}






