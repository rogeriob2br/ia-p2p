//! Configuration for the Inference Pool.
//!
//! Configuration can be loaded from:
//! - Environment variables (prefixed with `POOL_`)
//! - Configuration file (config.toml)
//! - Command line arguments

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Main configuration for the Inference Pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Pool identity and metadata
    pub pool: PoolIdentity,

    /// HTTP API server configuration
    pub api: ApiConfig,

    /// P2P network configuration (for receiving orchestration events)
    pub p2p: P2pConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Registry configuration
    pub registry: RegistryConfig,

    /// Router configuration
    pub router: RouterConfig,

    /// Billing configuration
    pub billing: BillingConfig,

    /// Lightning Network configuration
    pub lightning: LightningConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Pool identity and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolIdentity {
    /// Unique pool identifier
    pub pool_id: String,

    /// Human-readable pool name
    pub name: String,

    /// Pool description
    pub description: String,

    /// Pool operator name/organization
    pub operator: String,

    /// Contact email
    pub contact_email: Option<String>,

    /// Pool version
    pub version: String,
}

/// HTTP API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Address to listen on
    pub listen_addr: SocketAddr,

    /// Enable TLS
    pub tls_enabled: bool,

    /// TLS certificate path
    pub tls_cert_path: Option<PathBuf>,

    /// TLS key path
    pub tls_key_path: Option<PathBuf>,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,

    /// Enable CORS
    pub cors_enabled: bool,

    /// CORS allowed origins (empty = all)
    pub cors_origins: Vec<String>,
}

/// P2P network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    /// Enable P2P listener
    pub enabled: bool,

    /// P2P listen addresses
    pub listen_addrs: Vec<String>,

    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,

    /// Orchestration events topic
    pub orchestration_topic: String,

    /// Heartbeat topic
    pub heartbeat_topic: String,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication (set to false for development)
    pub enabled: bool,

    /// API key prefix for identification
    pub key_prefix: String,

    /// Default rate limit: requests per minute
    pub default_rate_limit_rpm: u32,

    /// Default rate limit: tokens per day
    pub default_tokens_per_day: u64,

    /// Admin API keys (for pool administration)
    pub admin_keys: Vec<String>,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Maximum number of pool members
    pub max_members: usize,

    /// Heartbeat interval expected from members (ms)
    pub heartbeat_interval_ms: u64,

    /// Heartbeat timeout before marking inactive (ms)
    pub heartbeat_timeout_ms: u64,

    /// Number of missed heartbeats before marking inactive
    pub max_missed_heartbeats: u32,

    /// Enable auto-deregistration of unhealthy nodes
    pub auto_deregister: bool,

    /// Minimum success rate (0.0 - 1.0) to stay registered
    pub min_success_rate: f64,

    /// Hours before deregistering unhealthy nodes
    pub unhealthy_threshold_hours: u64,
}

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Default routing strategy
    pub default_strategy: RoutingStrategyConfig,

    /// Maximum retries on failure
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,

    /// Job execution timeout in milliseconds
    pub job_timeout_ms: u64,

    /// Enable network map consultation for routing
    pub use_network_map: bool,

    /// Weight for load in scoring (0.0 - 1.0)
    pub load_weight: f64,

    /// Weight for latency in scoring (0.0 - 1.0)
    pub latency_weight: f64,

    /// Weight for success rate in scoring (0.0 - 1.0)
    pub success_weight: f64,
}

/// Routing strategy configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategyConfig {
    /// Simple round-robin
    RoundRobin,
    /// Route to least loaded member
    LeastLoaded,
    /// Route to member with lowest latency
    LowestLatency,
    /// Weighted score based on multiple factors
    WeightedScore,
    /// Use context groups from network map
    ContextAware,
}

/// Billing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    /// Enable billing system
    pub enabled: bool,

    /// Base fee per 1000 tokens (in default currency)
    pub base_fee_per_1k_tokens: f64,

    /// Default currency
    pub default_currency: String,

    /// Pool fee percentage (taken from each transaction)
    pub pool_fee_percent: f64,

    /// Billing period in hours (for invoice generation)
    pub billing_period_hours: u64,

    /// Enable instant payments via Lightning
    pub instant_payments: bool,
}

/// Lightning Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningConfig {
    /// Enable Lightning integration
    pub enabled: bool,

    /// Lightning backend type
    pub backend: LightningBackend,

    /// LND/CLN host
    pub host: String,

    /// LND/CLN port
    pub port: u16,

    /// Path to macaroon file (LND)
    pub macaroon_path: Option<PathBuf>,

    /// Path to TLS cert (LND)
    pub tls_cert_path: Option<PathBuf>,

    /// Default channel capacity in satoshis
    pub default_channel_capacity_sats: u64,

    /// Minimum local balance before rebalancing (sats)
    pub min_local_balance_sats: u64,

    /// Auto-rebalance channels
    pub auto_rebalance: bool,
}

/// Lightning backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightningBackend {
    /// Lightning Network Daemon
    Lnd,
    /// Core Lightning
    Cln,
    /// Mock backend for testing
    Mock,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, pretty)
    pub format: String,

    /// Log to file
    pub file_enabled: bool,

    /// Log file path
    pub file_path: Option<PathBuf>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            pool: PoolIdentity::default(),
            api: ApiConfig::default(),
            p2p: P2pConfig::default(),
            auth: AuthConfig::default(),
            registry: RegistryConfig::default(),
            router: RouterConfig::default(),
            billing: BillingConfig::default(),
            lightning: LightningConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for PoolIdentity {
    fn default() -> Self {
        Self {
            pool_id: uuid::Uuid::new_v4().to_string(),
            name: "Inference Pool".to_string(),
            description: "Decentralized AI inference pool".to_string(),
            operator: "Pool Operator".to_string(),
            contact_email: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".parse().unwrap(),
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            max_connections: 10000,
            request_timeout_ms: 300_000, // 5 minutes
            cors_enabled: true,
            cors_origins: vec![],
        }
    }
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addrs: vec!["/ip4/0.0.0.0/tcp/9000".to_string()],
            bootstrap_nodes: vec![],
            orchestration_topic: "/pool/orchestration/1.0.0".to_string(),
            heartbeat_topic: "/pool/heartbeat/1.0.0".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_prefix: "sk-pool-".to_string(),
            default_rate_limit_rpm: 60,
            default_tokens_per_day: 1_000_000,
            admin_keys: vec![],
        }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_members: 1000,
            heartbeat_interval_ms: 30_000, // 30 seconds
            heartbeat_timeout_ms: 90_000,  // 90 seconds (3 missed)
            max_missed_heartbeats: 3,
            auto_deregister: true,
            min_success_rate: 0.90, // 90%
            unhealthy_threshold_hours: 48,
        }
    }
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_strategy: RoutingStrategyConfig::WeightedScore,
            max_retries: 3,
            retry_delay_ms: 1000,
            job_timeout_ms: 120_000, // 2 minutes
            use_network_map: true,
            load_weight: 0.3,
            latency_weight: 0.3,
            success_weight: 0.4,
        }
    }
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_fee_per_1k_tokens: 0.001, // $0.001 per 1k tokens
            default_currency: "USD".to_string(),
            pool_fee_percent: 10.0, // 10% pool fee
            billing_period_hours: 720, // 30 days
            instant_payments: true,
        }
    }
}

impl Default for LightningConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default until configured
            backend: LightningBackend::Mock,
            host: "localhost".to_string(),
            port: 10009,
            macaroon_path: None,
            tls_cert_path: None,
            default_channel_capacity_sats: 1_000_000, // 0.01 BTC
            min_local_balance_sats: 100_000,          // 0.001 BTC
            auto_rebalance: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file_enabled: false,
            file_path: None,
        }
    }
}

impl PoolConfig {
    /// Get the pool_id shortcut
    pub fn pool_id(&self) -> &str {
        &self.pool.pool_id
    }

    /// Convert to PoolInfo for API responses
    pub fn to_pool_info(&self) -> crate::models::PoolInfo {
        use crate::models::{FeeStructure, PoolInfo, PoolPolicies, SlaTier, SlaTierInfo};
        use std::collections::HashMap;

        PoolInfo {
            pool_id: self.pool.pool_id.clone(),
            name: self.pool.name.clone(),
            description: self.pool.description.clone(),
            operator: self.pool.operator.clone(),
            fee_structure: FeeStructure {
                base_fee_per_1k_tokens: self.billing.base_fee_per_1k_tokens,
                currency: self.billing.default_currency.clone(),
                tier_multipliers: HashMap::from([
                    ("free".to_string(), 1.0),
                    ("basic".to_string(), 1.0),
                    ("pro".to_string(), 0.9),
                    ("enterprise".to_string(), 0.8),
                ]),
            },
            sla_tiers: vec![
                SlaTierInfo {
                    tier: SlaTier::Free,
                    max_latency_ms: 5000,
                    availability_sla: 95.0,
                    priority: 1,
                    price_multiplier: 1.0,
                },
                SlaTierInfo {
                    tier: SlaTier::Basic,
                    max_latency_ms: 3000,
                    availability_sla: 99.0,
                    priority: 2,
                    price_multiplier: 1.0,
                },
                SlaTierInfo {
                    tier: SlaTier::Pro,
                    max_latency_ms: 1500,
                    availability_sla: 99.5,
                    priority: 3,
                    price_multiplier: 0.9,
                },
                SlaTierInfo {
                    tier: SlaTier::Enterprise,
                    max_latency_ms: 1000,
                    availability_sla: 99.9,
                    priority: 4,
                    price_multiplier: 0.8,
                },
            ],
            policies: PoolPolicies::default(),
            registration_endpoint: format!("http://{}/v1/pool/register", self.api.listen_addr),
            version: self.pool.version.clone(),
        }
    }

    /// Load configuration from file
    pub fn from_file(path: &std::path::Path) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path))
            .add_source(
                config::Environment::with_prefix("POOL")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }

    /// Load configuration from default locations
    pub fn load() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            // Start with defaults
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::File::with_name("config.local").required(false))
            // Override with environment variables
            .add_source(
                config::Environment::with_prefix("POOL")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate API config
        if self.api.request_timeout_ms == 0 {
            errors.push("api.request_timeout_ms must be greater than 0".to_string());
        }

        // Validate router weights
        let weight_sum = self.router.load_weight
            + self.router.latency_weight
            + self.router.success_weight;
        if (weight_sum - 1.0).abs() > 0.001 {
            errors.push(format!(
                "Router weights must sum to 1.0, got {}",
                weight_sum
            ));
        }

        // Validate billing
        if self.billing.pool_fee_percent < 0.0 || self.billing.pool_fee_percent > 100.0 {
            errors.push("billing.pool_fee_percent must be between 0 and 100".to_string());
        }

        // Validate registry
        if self.registry.min_success_rate < 0.0 || self.registry.min_success_rate > 1.0 {
            errors.push("registry.min_success_rate must be between 0.0 and 1.0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PoolConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_weights() {
        let mut config = PoolConfig::default();
        config.router.load_weight = 0.5;
        config.router.latency_weight = 0.5;
        config.router.success_weight = 0.5; // Sum = 1.5

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("weights must sum"));
    }

    #[test]
    fn test_config_validation_pool_fee() {
        let mut config = PoolConfig::default();
        config.billing.pool_fee_percent = 150.0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("pool_fee_percent"));
    }

    #[test]
    fn test_routing_strategy_serialization() {
        let strategy = RoutingStrategyConfig::WeightedScore;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"weighted_score\"");

        let deserialized: RoutingStrategyConfig =
            serde_json::from_str("\"context_aware\"").unwrap();
        assert_eq!(deserialized, RoutingStrategyConfig::ContextAware);
    }

    #[test]
    fn test_lightning_backend_serialization() {
        let backend = LightningBackend::Lnd;
        let json = serde_json::to_string(&backend).unwrap();
        assert_eq!(json, "\"lnd\"");
    }
}

