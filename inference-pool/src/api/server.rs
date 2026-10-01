//! HTTP API Server.

use crate::config::PoolConfig;
use crate::registry::PoolRegistry;
use crate::router::JobRouter;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::handlers;

/// Shared API state
pub struct ApiState<R, J>
where
    R: PoolRegistry,
    J: JobRouter,
{
    /// Pool registry
    pub registry: Arc<R>,

    /// Job router
    pub router: Arc<J>,

    /// Pool configuration
    pub config: Arc<PoolConfig>,
}

impl<R, J> Clone for ApiState<R, J>
where
    R: PoolRegistry,
    J: JobRouter,
{
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            router: Arc::clone(&self.router),
            config: Arc::clone(&self.config),
        }
    }
}

impl<R, J> ApiState<R, J>
where
    R: PoolRegistry,
    J: JobRouter,
{
    /// Create new API state
    pub fn new(registry: Arc<R>, router: Arc<J>, config: PoolConfig) -> Self {
        Self {
            registry,
            router,
            config: Arc::new(config),
        }
    }
}

/// Create the API router
pub fn create_router<R, J>(state: ApiState<R, J>) -> Router
where
    R: PoolRegistry + 'static,
    J: JobRouter + 'static,
{
    Router::new()
        // OpenAI-compatible endpoints
        .route("/v1/chat/completions", post(handlers::chat_completions::<R, J>))
        .route("/v1/embeddings", post(handlers::embeddings::<R, J>))
        .route("/v1/models", get(handlers::list_models::<R, J>))
        // Pool management endpoints
        .route("/v1/pool/status", get(handlers::pool_status::<R, J>))
        .route("/v1/pool/register", post(handlers::register_node::<R, J>))
        // Health and utility endpoints
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::readiness_check::<R, J>))
        // Add layers
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        // Add state
        .with_state(state)
}

/// API Server wrapper
pub struct ApiServer {
    /// Listen address
    addr: String,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    /// Start the server
    pub async fn start<R, J>(self, state: ApiState<R, J>) -> std::io::Result<()>
    where
        R: PoolRegistry + 'static,
        J: JobRouter + 'static,
    {
        let router = create_router(state);

        tracing::info!(addr = %self.addr, "Starting API server");

        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

