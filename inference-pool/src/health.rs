//! Health Check Service for production monitoring.
//!
//! Provides endpoints for:
//! - Liveness check (/health/live)
//! - Readiness check (/health/ready)
//! - Detailed health (/health)
//!
//! ## Example
//!
//! ```ignore
//! use inference_pool::health::{HealthService, HealthCheck, ComponentHealth};
//!
//! let health = HealthService::new();
//! health.register_check("database", db_check);
//! health.register_check("lightning", ln_check);
//!
//! let status = health.check_all().await;
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Health Status
// ============================================================================

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All components healthy
    Healthy,
    /// Some components degraded but functional
    Degraded,
    /// System unhealthy
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Healthy
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

// ============================================================================
// Component Health
// ============================================================================

/// Health status of a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,

    /// Health status
    pub status: HealthStatus,

    /// Human-readable message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,

    /// Additional details
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub details: HashMap<String, serde_json::Value>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,
}

impl ComponentHealth {
    /// Create a healthy component
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            response_time_ms: None,
            details: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Create a degraded component
    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            response_time_ms: None,
            details: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Create an unhealthy component
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            response_time_ms: None,
            details: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Add response time
    pub fn with_response_time(mut self, ms: u64) -> Self {
        self.response_time_ms = Some(ms);
        self
    }

    /// Add a detail
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.details.insert(key.into(), v);
        }
        self
    }
}

// ============================================================================
// Full Health Response
// ============================================================================

/// Complete health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: HealthStatus,

    /// Service version
    pub version: String,

    /// Server uptime in seconds
    pub uptime_secs: u64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,

    /// System metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<SystemMetrics>,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: 0,
            timestamp: Utc::now(),
            components: HashMap::new(),
            metrics: None,
        }
    }
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    /// Active connections
    pub active_connections: u64,

    /// Requests per second
    pub requests_per_second: f64,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Total requests served
    pub total_requests: u64,

    /// Memory usage in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,

    /// CPU usage percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f64>,
}

// ============================================================================
// Health Check Trait
// ============================================================================

/// Trait for health check implementations
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check
    async fn check(&self) -> ComponentHealth;

    /// Get component name
    fn name(&self) -> &str;

    /// Whether this check is required for readiness
    fn required_for_ready(&self) -> bool {
        true
    }
}

// ============================================================================
// Health Service
// ============================================================================

/// Health service for managing checks
pub struct HealthService {
    /// Registered health checks
    checks: RwLock<Vec<Arc<dyn HealthCheck>>>,

    /// Service start time
    start_time: DateTime<Utc>,

    /// Cached health response
    cached_response: RwLock<Option<HealthResponse>>,

    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

impl HealthService {
    /// Create a new health service
    pub fn new() -> Self {
        Self {
            checks: RwLock::new(Vec::new()),
            start_time: Utc::now(),
            cached_response: RwLock::new(None),
            cache_ttl_secs: 5, // 5 second cache
        }
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, secs: u64) -> Self {
        self.cache_ttl_secs = secs;
        self
    }

    /// Register a health check
    pub async fn register<C: HealthCheck + 'static>(&self, check: C) {
        let mut checks = self.checks.write().await;
        checks.push(Arc::new(check));
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        (Utc::now() - self.start_time).num_seconds().max(0) as u64
    }

    /// Perform liveness check (is the service running?)
    pub async fn check_live(&self) -> bool {
        true // If this code is running, we're alive
    }

    /// Perform readiness check (can we serve traffic?)
    pub async fn check_ready(&self) -> bool {
        let checks = self.checks.read().await;

        for check in checks.iter() {
            if check.required_for_ready() {
                let result = check.check().await;
                if result.status == HealthStatus::Unhealthy {
                    return false;
                }
            }
        }

        true
    }

    /// Perform full health check
    pub async fn check_all(&self) -> HealthResponse {
        // Check cache
        {
            let cached = self.cached_response.read().await;
            if let Some(ref response) = *cached {
                let age = (Utc::now() - response.timestamp).num_seconds();
                if age < self.cache_ttl_secs as i64 {
                    return response.clone();
                }
            }
        }

        // Perform checks
        let checks = self.checks.read().await;
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for check in checks.iter() {
            let start = std::time::Instant::now();
            let mut result = check.check().await;
            result.response_time_ms = Some(start.elapsed().as_millis() as u64);

            // Update overall status
            match result.status {
                HealthStatus::Unhealthy => {
                    if check.required_for_ready() {
                        overall_status = HealthStatus::Unhealthy;
                    } else if overall_status != HealthStatus::Unhealthy {
                        overall_status = HealthStatus::Degraded;
                    }
                }
                HealthStatus::Degraded => {
                    if overall_status == HealthStatus::Healthy {
                        overall_status = HealthStatus::Degraded;
                    }
                }
                HealthStatus::Healthy => {}
            }

            components.insert(result.name.clone(), result);
        }

        let response = HealthResponse {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: self.uptime_secs(),
            timestamp: Utc::now(),
            components,
            metrics: None,
        };

        // Update cache
        {
            let mut cached = self.cached_response.write().await;
            *cached = Some(response.clone());
        }

        response
    }
}

impl Default for HealthService {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in Health Checks
// ============================================================================

/// Simple ping check
pub struct PingCheck {
    name: String,
}

impl PingCheck {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl HealthCheck for PingCheck {
    async fn check(&self) -> ComponentHealth {
        ComponentHealth::healthy(&self.name)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Database health check
pub struct DatabaseHealthCheck {
    pool: Arc<crate::database::DatabasePool>,
}

impl DatabaseHealthCheck {
    pub fn new(pool: Arc<crate::database::DatabasePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let db_health = self.pool.health_check().await;

        if db_health.connected {
            ComponentHealth::healthy("database")
                .with_detail("active_connections", db_health.active_connections)
                .with_detail("idle_connections", db_health.idle_connections)
                .with_detail("schema_version", db_health.schema_version)
        } else {
            ComponentHealth::unhealthy(
                "database",
                db_health.error.unwrap_or_else(|| "Connection failed".to_string()),
            )
        }
    }

    fn name(&self) -> &str {
        "database"
    }
}

// ============================================================================
// Axum Handlers
// ============================================================================

/// Create health check router
pub fn health_routes(service: Arc<HealthService>) -> axum::Router {
    use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

    async fn live_handler(State(service): State<Arc<HealthService>>) -> StatusCode {
        if service.check_live().await {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        }
    }

    async fn ready_handler(State(service): State<Arc<HealthService>>) -> StatusCode {
        if service.check_ready().await {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        }
    }

    async fn health_handler(
        State(service): State<Arc<HealthService>>,
    ) -> (StatusCode, Json<HealthResponse>) {
        let response = service.check_all().await;
        let status = match response.status {
            HealthStatus::Healthy => StatusCode::OK,
            HealthStatus::Degraded => StatusCode::OK,
            HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, Json(response))
    }

    Router::new()
        .route("/live", get(live_handler))
        .route("/ready", get(ready_handler))
        .route("/", get(health_handler))
        .with_state(service)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_builders() {
        let healthy = ComponentHealth::healthy("test");
        assert_eq!(healthy.status, HealthStatus::Healthy);

        let degraded = ComponentHealth::degraded("test", "Slow response");
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert!(degraded.message.is_some());

        let unhealthy = ComponentHealth::unhealthy("test", "Connection failed");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_component_health_with_details() {
        let health = ComponentHealth::healthy("db")
            .with_response_time(50)
            .with_detail("connections", 10u32);

        assert_eq!(health.response_time_ms, Some(50));
        assert!(health.details.contains_key("connections"));
    }

    #[tokio::test]
    async fn test_health_service_empty() {
        let service = HealthService::new();

        assert!(service.check_live().await);
        assert!(service.check_ready().await);

        let response = service.check_all().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(response.components.is_empty());
    }

    #[tokio::test]
    async fn test_health_service_with_checks() {
        let service = HealthService::new();
        service.register(PingCheck::new("ping")).await;

        let response = service.check_all().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.components.len(), 1);
        assert!(response.components.contains_key("ping"));
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[tokio::test]
    async fn test_uptime() {
        let service = HealthService::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert!(service.uptime_secs() >= 0); // At least 0 seconds
    }
}


