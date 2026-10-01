//! Axum authentication middleware.

use super::{ApiKeyInfo, AuthStorage, RateLimiter};
use crate::error::PoolError;
use crate::models::SlaTier;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Client information extracted from authentication
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Key ID
    pub key_id: String,

    /// Client ID
    pub client_id: String,

    /// Client name
    pub client_name: String,

    /// SLA tier
    pub tier: SlaTier,
}

impl From<&ApiKeyInfo> for ClientInfo {
    fn from(info: &ApiKeyInfo) -> Self {
        Self {
            key_id: info.key_id.clone(),
            client_id: info.client_id.clone(),
            client_name: info.client_name.clone(),
            tier: info.tier,
        }
    }
}

/// Authentication state for middleware
#[derive(Clone)]
pub struct AuthState<S: AuthStorage, R: RateLimiter> {
    /// Auth storage
    pub storage: Arc<S>,

    /// Rate limiter
    pub rate_limiter: Arc<R>,

    /// Whether auth is enabled
    pub enabled: bool,
}

impl<S: AuthStorage, R: RateLimiter> AuthState<S, R> {
    /// Create new auth state
    pub fn new(storage: Arc<S>, rate_limiter: Arc<R>, enabled: bool) -> Self {
        Self {
            storage,
            rate_limiter,
            enabled,
        }
    }
}

/// Extract API key from Authorization header
pub fn extract_api_key(req: &Request) -> Option<String> {
    req.headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.to_lowercase().starts_with("bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        })
}

/// Authentication middleware function
pub async fn auth_middleware<S: AuthStorage + 'static, R: RateLimiter + 'static>(
    State(state): State<AuthState<S, R>>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, axum::Json<crate::error::ErrorResponse>)> {
    // Skip auth if disabled
    if !state.enabled {
        // Insert a default client info for testing
        req.extensions_mut().insert(ClientInfo {
            key_id: "test-key".to_string(),
            client_id: "test-client".to_string(),
            client_name: "Test Client".to_string(),
            tier: SlaTier::Enterprise,
        });
        return Ok(next.run(req).await);
    }

    // Extract API key
    let api_key = match extract_api_key(&req) {
        Some(key) => key,
        None => {
            let error = PoolError::api_key_missing();
            return Err((error.status_code(), axum::Json(error.to_response())));
        }
    };

    // Validate key
    let key_info = match state.storage.validate_key(&api_key).await {
        Ok(info) => info,
        Err(e) => {
            return Err((e.status_code(), axum::Json(e.to_response())));
        }
    };

    // Check rate limit
    let rate_config: super::RateLimitConfig = (&key_info.rate_limit).into();
    if let Err(e) = state
        .rate_limiter
        .check_request(&key_info.client_id, &rate_config)
        .await
    {
        return Err((e.status_code(), axum::Json(e.to_response())));
    }

    // Insert client info into request extensions
    req.extensions_mut().insert(ClientInfo::from(&key_info));

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_api_key() {
        // Test with valid Bearer token
        let req = Request::builder()
            .header("authorization", "Bearer sk-pool-test123")
            .body(axum::body::Body::empty())
            .unwrap();

        let key = extract_api_key(&req);
        assert_eq!(key, Some("sk-pool-test123".to_string()));

        // Test without header
        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let key = extract_api_key(&req);
        assert_eq!(key, None);

        // Test with wrong prefix
        let req = Request::builder()
            .header("authorization", "Basic abc123")
            .body(axum::body::Body::empty())
            .unwrap();
        let key = extract_api_key(&req);
        assert_eq!(key, None);
    }

    #[test]
    fn test_client_info_from_api_key_info() {
        let info = ApiKeyInfo {
            key_id: "key-1".to_string(),
            key_hash: "hash".to_string(),
            client_id: "client-1".to_string(),
            client_name: "Test Client".to_string(),
            tier: SlaTier::Pro,
            rate_limit: super::super::RateLimitConfig::default().into(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked_at: None,
            is_active: true,
            last_used_at: None,
            usage_count: 0,
        };

        let client_info = ClientInfo::from(&info);
        assert_eq!(client_info.key_id, "key-1");
        assert_eq!(client_info.client_id, "client-1");
        assert_eq!(client_info.tier, SlaTier::Pro);
    }
}
