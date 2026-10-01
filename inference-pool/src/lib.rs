//! # Inference Pool
//!
//! Layer 2 component for the decentralized AI P2P network.
//! Aggregates nodes and exposes an OpenAI-compatible API for easy integration
//! with frameworks like LangChain, CrewAI, and n8n.
//!
//! ## Architecture
//!
//! The Inference Pool operates as a manual registration system:
//! - Node operators manually register after commercial agreement
//! - Pool monitors distributed orchestration (does not control it)
//! - Pool respects network decisions (adaptive consensus, group formation)
//! - Payments via direct Lightning channels
//!
//! ## Main Components
//!
//! - **API Server**: OpenAI-compatible HTTP endpoints (Axum)
//! - **Pool Registry**: Manages Pool_Members and their capabilities
//! - **Job Router**: Intelligent job distribution with retry logic
//! - **Auth Middleware**: API key validation and rate limiting
//! - **Network Map**: Visualizes network topology and roles
//! - **Orchestration Observer**: Receives orchestration events via P2P
//! - **Billing System**: Tickets, invoices, Lightning payments
//! - **Channel Manager**: Direct Lightning channels with nodes

pub mod config;
pub mod error;
pub mod models;

// Core components
pub mod heartbeat;
pub mod registry;

// Database persistence
pub mod database;

// Production readiness
pub mod health;
pub mod lifecycle;

// Components
pub mod api;
pub mod auth;
pub mod billing;
pub mod channel_manager;
pub mod control_plane;
pub mod network_map;
pub mod observer;
pub mod orchestrator;
pub mod router;

// Re-exports for convenience
pub use config::PoolConfig;
pub use error::{PoolError, PoolErrorType, PoolResult};
pub use heartbeat::{HeartbeatConfig, HeartbeatEvent, HeartbeatMonitor, HeartbeatMonitorTrait};
pub use registry::{InMemoryPoolRegistry, MemberRegistration, PoolRegistry};
pub use auth::{
    ApiKeyInfo, AuthState, AuthStorage, ClientInfo, InMemoryAuthStorage,
    RateLimit, RateLimitConfig, RateLimiter, RateLimiterImpl, auth_middleware, extract_api_key,
};
pub use database::{
    DatabaseConfig, DatabaseHealth, DatabasePool, DatabaseType,
    Migration, MigrationManager, MigrationRecord,
};
pub use health::{
    ComponentHealth, HealthCheck, HealthResponse, HealthService, HealthStatus,
    SystemMetrics, health_routes,
};
pub use lifecycle::{
    LifecycleManager, LifecycleState, ShutdownReason, StartupValidation, StartupValidator,
};
pub use router::{
    JobRouter, JobRouterImpl, LeastLoadedStrategy, PerformanceStrategy, RouterStats,
    RoundRobinStrategy, RoutingResult, RoutingStrategy,
};
pub use api::{ApiServer, ApiState, create_router};
pub use observer::{NetworkObserver, NetworkObserverConfig, ObserverEvent, OrchestrationObserver};
pub use billing::{
    BillingService, BillingTicket, Currency, InMemoryBillingService, Invoice, InvoiceStatus,
    MemberEarnings, Payment, PaymentMethod, PaymentStatus, TicketStatus, UsageSummary,
};
pub use channel_manager::{
    Channel, ChannelConfig, ChannelEvent, ChannelManager, ChannelState, ChannelStats,
    ClnChannelManager, ClnClient, ClnConfig, InMemoryChannelManager, LightningBackend,
    LndChannelManager, LndClient, LndConfig, PaymentRequest, PaymentResult, PaymentState,
};

