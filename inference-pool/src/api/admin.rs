//! Admin API endpoints for pool management.
//!
//! Protected endpoints for pool operators:
//! - Pool statistics
//! - Member management  
//! - API key management
//! - Channel management
//! - System status

use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::database::DatabasePool;
use crate::health::HealthService;
use crate::lifecycle::LifecycleManager;

// ============================================================================
// Admin State
// ============================================================================

/// Admin API state
#[derive(Clone)]
pub struct AdminState {
    pub db: Arc<DatabasePool>,
    pub health: Arc<HealthService>,
    pub lifecycle: Arc<LifecycleManager>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Pool statistics response
#[derive(Debug, Serialize)]
pub struct PoolStats {
    pub pool_name: String,
    pub version: String,
    pub uptime_secs: u64,
    pub start_time: DateTime<Utc>,
    pub environment: String,
    pub members: MemberStats,
    pub api_keys: ApiKeyStats,
    pub billing: BillingStats,
    pub channels: ChannelStats,
}

/// Member statistics
#[derive(Debug, Serialize, Default)]
pub struct MemberStats {
    pub total: u32,
    pub active: u32,
    pub inactive: u32,
    pub draining: u32,
}

/// API key statistics
#[derive(Debug, Serialize, Default)]
pub struct ApiKeyStats {
    pub total: u32,
    pub active: u32,
    pub revoked: u32,
}

/// Billing statistics
#[derive(Debug, Serialize, Default)]
pub struct BillingStats {
    pub total_tickets: u64,
    pub pending_amount_sats: u64,
    pub paid_amount_sats: u64,
}

/// Channel statistics
#[derive(Debug, Serialize, Default)]
pub struct ChannelStats {
    pub total: u32,
    pub active: u32,
    pub total_capacity_sats: u64,
    pub total_local_balance_sats: u64,
}

/// Admin API response wrapper
#[derive(Debug, Serialize)]
pub struct AdminResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> AdminResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

}

impl AdminResponse<()> {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: Utc::now(),
        }
    }
}

// ============================================================================
// Request Types
// ============================================================================

/// Create API key request
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub client_id: String,
    pub client_name: String,
    #[serde(default)]
    pub tier: Option<String>,
    pub expires_days: Option<u32>,
}

/// Create API key response
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub key_id: String,
    pub api_key: String, // Only returned once on creation
    pub client_id: String,
    pub tier: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Shutdown request
#[derive(Debug, Deserialize)]
pub struct ShutdownRequest {
    pub reason: Option<String>,
    pub delay_secs: Option<u64>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Get pool statistics
async fn get_pool_stats(State(state): State<AdminState>) -> Json<AdminResponse<PoolStats>> {
    let members = match crate::database::queries::MemberQueries::count_by_status(state.db.pool())
        .await
    {
        Ok((total, active)) => MemberStats {
            total,
            active,
            inactive: total.saturating_sub(active),
            draining: 0,
        },
        Err(_) => MemberStats::default(),
    };

    let channels =
        match crate::database::queries::ChannelQueries::get_stats(state.db.pool()).await {
            Ok(stats) => ChannelStats {
                total: stats.total_channels,
                active: stats.active_channels,
                total_capacity_sats: stats.total_capacity_sats,
                total_local_balance_sats: stats.total_local_balance_sats,
            },
            Err(_) => ChannelStats::default(),
        };

    let stats = PoolStats {
        pool_name: "inference-pool".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.lifecycle.uptime_secs(),
        start_time: state.lifecycle.start_time(),
        environment: "development".to_string(),
        members,
        api_keys: ApiKeyStats::default(),
        billing: BillingStats::default(),
        channels,
    };

    Json(AdminResponse::success(stats))
}

/// Get system health
async fn get_system_health(
    State(state): State<AdminState>,
) -> Json<crate::health::HealthResponse> {
    Json(state.health.check_all().await)
}

/// List all members
async fn list_members(
    State(state): State<AdminState>,
) -> Result<Json<AdminResponse<Vec<MemberSummary>>>, (StatusCode, Json<AdminResponse<()>>)> {
    match crate::database::queries::MemberQueries::get_all(state.db.pool()).await {
        Ok(members) => {
            let summaries: Vec<MemberSummary> = members
                .into_iter()
                .map(|m| MemberSummary {
                    member_id: m.member_id,
                    node_id: m.node_id,
                    status: format!("{:?}", m.status),
                    models_count: m.models.len(),
                    last_heartbeat: m.last_heartbeat,
                    registered_at: m.registered_at,
                })
                .collect();
            Ok(Json(AdminResponse::success(summaries)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AdminResponse::error(e.to_string())),
        )),
    }
}

/// Member summary for listing
#[derive(Debug, Serialize)]
pub struct MemberSummary {
    pub member_id: String,
    pub node_id: String,
    pub status: String,
    pub models_count: usize,
    pub last_heartbeat: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
}

/// Get member details
async fn get_member(
    State(state): State<AdminState>,
    axum::extract::Path(member_id): axum::extract::Path<String>,
) -> Result<
    Json<AdminResponse<crate::models::PoolMember>>,
    (StatusCode, Json<AdminResponse<()>>),
> {
    match crate::database::queries::MemberQueries::get_by_id(state.db.pool(), &member_id).await {
        Ok(Some(member)) => Ok(Json(AdminResponse::success(member))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(AdminResponse::error(format!(
                "Member {} not found",
                member_id
            ))),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AdminResponse::error(e.to_string())),
        )),
    }
}

/// Remove a member
async fn remove_member(
    State(state): State<AdminState>,
    axum::extract::Path(member_id): axum::extract::Path<String>,
) -> Result<Json<AdminResponse<()>>, (StatusCode, Json<AdminResponse<()>>)> {
    match crate::database::queries::MemberQueries::delete(state.db.pool(), &member_id).await {
        Ok(true) => Ok(Json(AdminResponse {
            success: true,
            data: Some(()),
            error: None,
            timestamp: Utc::now(),
        })),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(AdminResponse::error(format!(
                "Member {} not found",
                member_id
            ))),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AdminResponse::error(e.to_string())),
        )),
    }
}

/// List channels
async fn list_channels(
    State(state): State<AdminState>,
) -> Result<
    Json<AdminResponse<Vec<crate::channel_manager::Channel>>>,
    (StatusCode, Json<AdminResponse<()>>),
> {
    match crate::database::queries::ChannelQueries::get_all(state.db.pool()).await {
        Ok(channels) => Ok(Json(AdminResponse::success(channels))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AdminResponse::error(e.to_string())),
        )),
    }
}

/// Trigger graceful shutdown
async fn trigger_shutdown(
    State(state): State<AdminState>,
    Json(req): Json<ShutdownRequest>,
) -> Json<AdminResponse<String>> {
    let reason = req.reason.unwrap_or_else(|| "Admin request".to_string());

    // Schedule shutdown
    let lifecycle = state.lifecycle.clone();
    let delay = req.delay_secs.unwrap_or(5);

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
        lifecycle
            .shutdown(crate::lifecycle::ShutdownReason::AdminRequest)
            .await;
    });

    Json(AdminResponse::success(format!(
        "Shutdown scheduled in {} seconds: {}",
        delay, reason
    )))
}

/// Database health
async fn get_db_health(
    State(state): State<AdminState>,
) -> Json<AdminResponse<crate::database::DatabaseHealth>> {
    let health = state.db.health_check().await;
    Json(AdminResponse::success(health))
}

// ============================================================================
// Router
// ============================================================================

/// Create admin API router
pub fn admin_routes(state: AdminState) -> Router {
    Router::new()
        // Pool operations
        .route("/stats", get(get_pool_stats))
        .route("/health", get(get_system_health))
        .route("/db/health", get(get_db_health))
        // Member management
        .route("/members", get(list_members))
        .route("/members/:member_id", get(get_member))
        .route("/members/:member_id", delete(remove_member))
        // Channel management
        .route("/channels", get(list_channels))
        // System operations
        .route("/shutdown", post(trigger_shutdown))
        .with_state(state)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_response_success() {
        let response = AdminResponse::success("test");
        assert!(response.success);
        assert_eq!(response.data, Some("test"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_admin_response_error() {
        let response: AdminResponse<()> = AdminResponse::error("Something failed");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Something failed".to_string()));
    }

    #[test]
    fn test_create_api_key_request_deserialize() {
        let json = r#"{"client_id": "c1", "client_name": "Test"}"#;
        let req: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.client_id, "c1");
        assert!(req.tier.is_none());
    }
}

