# Design Document: Pool Production Readiness

## Overview

Este documento especifica o design de features que tornam a Inference Pool pronta para produção. O design complementa a spec `api-integration` adicionando persistência, observabilidade, configuração robusta e APIs administrativas.

**Princípios**:
1. **Simplicidade**: Adicionar apenas o essencial para produção
2. **Flexibilidade**: Suportar desenvolvimento (SQLite) e produção (PostgreSQL)
3. **Observabilidade**: Logs, health checks e métricas
4. **Operabilidade**: APIs administrativas para operação
5. **Robustez**: Validação, graceful shutdown, error handling

## Architecture

```mermaid
graph TB
    subgraph "Production Features"
        CONFIG[Configuration Manager]
        DB[Database Layer]
        HEALTH[Health Check]
        LOG[Structured Logging]
        ADMIN[Admin API]
        SHUTDOWN[Shutdown Manager]
    end
    
    subgraph "Core Pool (from api-integration)"
        API[API Server]
        REG[Registry]
        BILL[Billing]
        CHAN[Channel Manager]
    end
    
    CONFIG --> API
    CONFIG --> DB
    
    DB --> REG
    DB --> BILL
    
    HEALTH --> REG
    HEALTH --> DB
    HEALTH --> CHAN
    
    LOG --> API
    LOG --> REG
    LOG --> BILL
    
    ADMIN --> REG
    ADMIN --> BILL
    
    SHUTDOWN --> API
    SHUTDOWN --> CHAN
    SHUTDOWN --> DB
```

## Components and Interfaces

### 1. Database Layer

**Por quê**: Persistir dados críticos (registry, billing, API keys) para sobreviver reinicializações.

**Onde**: `inference-pool/src/database/`

Abstração sobre SQLite e PostgreSQL com migrations.

```rust
/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type: "sqlite" or "postgres"
    pub db_type: DatabaseType,
    
    /// Connection string
    /// SQLite: "sqlite:pool.db" or ":memory:"
    /// PostgreSQL: "postgresql://user:pass@host/db"
    pub url: String,
    
    /// Auto-run migrations on startup
    pub auto_migrate: bool,
    
    /// Connection pool size
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

/// Database connection pool
pub struct DatabasePool {
    pool: sqlx::Pool<sqlx::Any>,
    db_type: DatabaseType,
}

impl DatabasePool {
    /// Create new database pool
    pub async fn new(config: DatabaseConfig) -> Result<Self>;
    
    /// Run pending migrations
    pub async fn migrate(&self) -> Result<()>;
    
    /// Get connection from pool
    pub async fn acquire(&self) -> Result<PoolConnection>;
    
    /// Check if database is healthy
    pub async fn health_check(&self) -> Result<DatabaseHealth>;
    
    /// Close all connections
    pub async fn close(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub migrations_current: bool,
    pub active_connections: u32,
}
```

### 2. Migration Manager

**Por quê**: Gerenciar evolução do schema de forma segura e versionada.

**Onde**: `inference-pool/src/database/migrations/`

```rust
/// Migration definition
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

/// Migration manager
pub struct MigrationManager {
    pool: DatabasePool,
}

impl MigrationManager {
    /// Get current schema version
    pub async fn current_version(&self) -> Result<i64>;
    
    /// Get pending migrations
    pub async fn pending_migrations(&self) -> Result<Vec<Migration>>;
    
    /// Run all pending migrations
    pub async fn migrate_up(&self) -> Result<()>;
    
    /// Rollback last migration
    pub async fn migrate_down(&self) -> Result<()>;
    
    /// Get migration history
    pub async fn migration_history(&self) -> Result<Vec<MigrationRecord>>;
}

#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: i64,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub success: bool,
}
```

**Migrations** (em `inference-pool/src/database/migrations/`):

```sql
-- 001_initial_schema.sql
CREATE TABLE IF NOT EXISTS pool_members (
    node_id TEXT PRIMARY KEY,
    peer_id TEXT NOT NULL,
    capabilities JSON NOT NULL,
    models JSON NOT NULL,
    status TEXT NOT NULL,
    lightning_pubkey TEXT,
    registered_at TIMESTAMP NOT NULL,
    last_heartbeat TIMESTAMP,
    metrics JSON
);

CREATE TABLE IF NOT EXISTS api_keys (
    key_id TEXT PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,
    client_id TEXT NOT NULL,
    tier TEXT NOT NULL,
    rate_limit JSON NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,
    revoked_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS billing_tickets (
    ticket_id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    model TEXT NOT NULL,
    tokens JSON NOT NULL,
    client_cost JSON NOT NULL,
    member_payment JSON NOT NULL,
    pool_fee JSON NOT NULL,
    payment_status TEXT NOT NULL,
    channel_id TEXT
);

CREATE TABLE IF NOT EXISTS invoices (
    invoice_id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    tickets JSON NOT NULL,
    subtotal JSON NOT NULL,
    tax JSON,
    total JSON NOT NULL,
    status TEXT NOT NULL,
    due_date TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS lightning_channels (
    channel_id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    node_pubkey TEXT NOT NULL,
    capacity_sats BIGINT NOT NULL,
    local_balance_sats BIGINT NOT NULL,
    remote_balance_sats BIGINT NOT NULL,
    status TEXT NOT NULL,
    opened_at TIMESTAMP NOT NULL,
    last_payment TIMESTAMP,
    total_payments BIGINT DEFAULT 0,
    total_sats_sent BIGINT DEFAULT 0
);

CREATE TABLE IF NOT EXISTS migrations (
    version BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TIMESTAMP NOT NULL,
    success BOOLEAN NOT NULL
);
```

### 3. Health Check Service

**Por quê**: Load balancers e monitoring precisam saber se a pool está saudável.

**Onde**: `inference-pool/src/health/`

```rust
/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthState,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: ComponentHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub database: ComponentStatus,
    pub lightning: ComponentStatus,
    pub registry: ComponentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: HealthState,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

/// Health check service
#[async_trait]
pub trait HealthCheckService: Send + Sync {
    /// Get current health status
    async fn check_health(&self) -> HealthStatus;
    
    /// Check specific component
    async fn check_component(&self, component: &str) -> ComponentStatus;
    
    /// Register health check for component
    fn register_check(&mut self, component: String, check: Box<dyn HealthCheck>);
}

/// Health check trait for components
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> ComponentStatus;
}
```

### 4. Structured Logging

**Por quê**: Logs estruturados permitem análise, alertas e debugging eficiente.

**Onde**: `inference-pool/src/logging/`

```rust
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{fmt, EnvFilter};

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: ERROR, WARN, INFO, DEBUG, TRACE
    pub level: String,
    
    /// Output format: "json" or "pretty"
    pub format: LogFormat,
    
    /// Include file and line numbers
    pub include_location: bool,
    
    /// Include thread IDs
    pub include_thread_id: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

/// Initialize logging
pub fn init_logging(config: LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    match config.format {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(filter)
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt()
                .pretty()
                .with_env_filter(filter)
                .init();
        }
    }
    
    Ok(())
}

/// Log events with context
/// Example usage:
/// info!(
///     node_id = %member.node_id,
///     models = ?member.models,
///     "Node registered successfully"
/// );
```

### 5. Configuration Manager

**Por quê**: Configuração flexível via arquivo + env vars para diferentes ambientes.

**Onde**: `inference-pool/src/config/`

```rust
/// Complete pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub pool: PoolInfo,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub lightning: LightningConfig,
    pub logging: LoggingConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub name: String,
    pub operator: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    /// Admin API token (required for admin endpoints)
    pub token: String,
    
    /// Enable admin API
    pub enabled: bool,
}

impl PoolConfig {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self> {
        // 1. Load from config.toml
        let mut config: PoolConfig = Self::load_from_file("config.toml")?;
        
        // 2. Override with environment variables
        config.apply_env_overrides()?;
        
        // 3. Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load from TOML file
    fn load_from_file(path: &str) -> Result<Self>;
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<()>;
    
    /// Validate configuration
    fn validate(&self) -> Result<()>;
    
    /// Get environment (development, staging, production)
    pub fn environment() -> Environment {
        std::env::var("POOL_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .parse()
            .unwrap_or(Environment::Development)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}
```

**config.toml example**:

```toml
[pool]
name = "My Inference Pool"
operator = "operator@example.com"
description = "A decentralized AI inference pool"

[server]
host = "0.0.0.0"
port = 8080
# TLS is optional for development, required for production
tls_cert = "${TLS_CERT_PATH}"
tls_key = "${TLS_KEY_PATH}"

[database]
# Use "sqlite" for development, "postgres" for production
db_type = "sqlite"
url = "sqlite:pool.db"
auto_migrate = true
max_connections = 10

[lightning]
implementation = "lnd"  # or "cln"
grpc_url = "${LND_GRPC_URL}"
macaroon_path = "${LND_MACAROON_PATH}"
tls_cert_path = "${LND_TLS_CERT_PATH}"

[logging]
level = "INFO"
format = "pretty"  # "json" for production
include_location = true
include_thread_id = false

[admin]
enabled = true
token = "${ADMIN_TOKEN}"
```

### 6. Shutdown Manager

**Por quê**: Desligamento ordenado previne perda de dados e conexões abruptas.

**Onde**: `inference-pool/src/shutdown/`

```rust
/// Shutdown manager
pub struct ShutdownManager {
    shutdown_tx: broadcast::Sender<()>,
    components: Vec<Box<dyn ShutdownHook>>,
}

impl ShutdownManager {
    /// Create new shutdown manager
    pub fn new() -> Self;
    
    /// Register component for shutdown
    pub fn register(&mut self, component: Box<dyn ShutdownHook>);
    
    /// Start listening for shutdown signals
    pub async fn listen_for_signals(&self);
    
    /// Initiate graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        info!("Initiating graceful shutdown");
        
        // 1. Stop accepting new requests
        info!("Stopping API server");
        self.shutdown_component("api_server").await?;
        
        // 2. Wait for in-flight requests (max 30s)
        info!("Waiting for in-flight requests");
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // 3. Close Lightning channels cooperatively
        info!("Closing Lightning channels");
        self.shutdown_component("channel_manager").await?;
        
        // 4. Flush pending data to database
        info!("Flushing data to database");
        self.shutdown_component("billing_system").await?;
        
        // 5. Close database connections
        info!("Closing database connections");
        self.shutdown_component("database").await?;
        
        info!("Graceful shutdown complete");
        Ok(())
    }
    
    async fn shutdown_component(&self, name: &str) -> Result<()>;
}

/// Shutdown hook trait
#[async_trait]
pub trait ShutdownHook: Send + Sync {
    async fn shutdown(&self) -> Result<()>;
}
```

### 7. Admin API

**Por quê**: Operadores precisam gerenciar API keys e nodes sem acessar banco diretamente.

**Onde**: `inference-pool/src/admin/`

```rust
/// Admin API endpoints
pub struct AdminApi {
    auth: Arc<dyn AuthMiddleware>,
    registry: Arc<dyn PoolRegistry>,
    billing: Arc<dyn BillingSystem>,
}

impl AdminApi {
    /// POST /v1/admin/keys/create
    pub async fn create_api_key(&self, req: CreateKeyRequest) -> Result<CreateKeyResponse>;
    
    /// POST /v1/admin/keys/revoke
    pub async fn revoke_api_key(&self, key_id: String) -> Result<()>;
    
    /// GET /v1/admin/keys
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyInfo>>;
    
    /// GET /v1/admin/members
    pub async fn list_members(&self) -> Result<Vec<MemberDetail>>;
    
    /// GET /v1/admin/members/{node_id}
    pub async fn get_member(&self, node_id: String) -> Result<MemberDetail>;
    
    /// POST /v1/admin/members/{node_id}/deregister
    pub async fn deregister_member(&self, node_id: String, reason: String) -> Result<()>;
    
    /// GET /v1/admin/stats
    pub async fn get_stats(&self) -> Result<PoolStats>;
    
    /// GET /v1/admin/config
    pub async fn get_config(&self) -> Result<ConfigResponse>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    pub client_id: String,
    pub tier: SlaTier,
    pub rate_limit: RateLimit,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    pub key_id: String,
    pub api_key: String,  // Only shown once
    pub client_id: String,
    pub tier: SlaTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberDetail {
    pub node_id: String,
    pub peer_id: String,
    pub status: MemberStatus,
    pub models: Vec<String>,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub metrics: MemberMetrics,
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_members: u32,
    pub active_members: u32,
    pub total_requests_24h: u64,
    pub total_tokens_24h: u64,
    pub total_revenue_24h: f64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub pool: PoolInfo,
    pub server: ServerConfig,
    pub database_type: String,
    pub lightning_implementation: String,
    // Secrets are NOT included
}
```

## Data Models

### Database Schema

See Migration 001 in Component 2 (Migration Manager) above.

## Correctness Properties

### Property 35: Database Persistence Round-Trip

*For any* pool member registered, persisting to database and then loading from database SHALL return equivalent member data.

**Validates: Requirements 1.2, 1.4**

### Property 36: Migration Idempotency

*For any* migration, running it multiple times SHALL produce the same final schema state.

**Validates: Requirements 2.2**

### Property 37: Health Check Reflects Reality

*For any* component failure, the health check endpoint SHALL report unhealthy status within 1 second.

**Validates: Requirements 3.2, 3.3**

### Property 38: Configuration Override Precedence

*For any* configuration value, environment variable SHALL override file value.

**Validates: Requirements 6.2**

### Property 39: Graceful Shutdown Completes

*For any* shutdown signal, all in-flight requests SHALL complete before database closes.

**Validates: Requirements 5.3, 5.5**

### Property 40: Admin API Requires Auth

*For any* admin endpoint request without valid admin token, the API SHALL return 401 Unauthorized.

**Validates: Requirements 8.6**

## Error Handling

All errors follow the same OpenAI-compatible format from `api-integration` spec.

Additional error types:
- `database_error`: Database connection or query failed
- `migration_error`: Migration execution failed
- `configuration_error`: Invalid configuration
- `shutdown_error`: Graceful shutdown failed

## Testing Strategy

### Unit Tests

- Database CRUD operations
- Migration up/down
- Health check logic
- Configuration loading and validation
- Shutdown sequence

### Property-Based Tests

- **Minimum 100 iterations** per property test
- Tag format: `Feature: pool-production-ready, Property N: <property_text>`

### Integration Tests

- Full startup with database
- Health check with component failures
- Graceful shutdown with in-flight requests
- Admin API with authentication
- Configuration loading from file + env vars

