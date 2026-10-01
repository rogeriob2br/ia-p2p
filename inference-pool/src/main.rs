//! Inference Pool - Layer 2 component for the decentralized AI P2P network.
//!
//! Aggregates nodes and exposes an OpenAI-compatible API.

use inference_pool::{
    api::{ApiServer, ApiState},
    config::PoolConfig,
    heartbeat::{HeartbeatConfig, HeartbeatMonitor},
    registry::InMemoryPoolRegistry,
    router::{JobRouterImpl, RoundRobinStrategy},
    PoolRegistry,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    init_tracing();

    tracing::info!("Starting Inference Pool v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = PoolConfig::default();

    // Log configuration summary
    tracing::info!(
        pool_id = %config.pool.pool_id,
        listen_addr = %config.api.listen_addr,
        "Configuration loaded"
    );

    // Validate configuration
    if let Err(errors) = config.validate() {
        for error in &errors {
            tracing::error!("Configuration error: {}", error);
        }
        anyhow::bail!("Invalid configuration");
    }

    // Create registry with registration keys
    let registry = Arc::new(InMemoryPoolRegistry::new());

    // Add a default registration key for development
    registry
        .add_registration_key("dev-registration-key".to_string())
        .await?;

    // Create job router
    let router = Arc::new(JobRouterImpl::new(
        Arc::clone(&registry),
        RoundRobinStrategy::new(),
    ));

    // Create heartbeat monitor
    let heartbeat_config = HeartbeatConfig {
        interval_ms: config.registry.heartbeat_interval_ms,
        timeout_ms: config.registry.heartbeat_timeout_ms,
        max_missed: config.registry.max_missed_heartbeats,
        check_interval_ms: config.registry.heartbeat_interval_ms / 2,
    };
    let _heartbeat_monitor = HeartbeatMonitor::new(heartbeat_config, Arc::clone(&registry));

    // TODO: Start heartbeat monitoring in background
    // heartbeat_monitor.start().await?;

    // Create API state
    let api_state = ApiState::new(
        Arc::clone(&registry),
        Arc::clone(&router),
        config.clone(),
    );

    // Start HTTP server
    let server = ApiServer::new(config.api.listen_addr.to_string());

    tracing::info!(
        addr = %config.api.listen_addr,
        "API server starting"
    );

    server.start(api_state).await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inference_pool=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
