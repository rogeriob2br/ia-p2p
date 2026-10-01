//! Database connection pool implementation.

use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Row};
use sqlx::any::AnyPoolOptions;
use std::time::Duration;

use crate::error::{PoolError, PoolResult};

// ============================================================================
// Configuration
// ============================================================================

/// Database type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    /// SQLite - for development and testing
    Sqlite,
    /// PostgreSQL - for production
    Postgres,
}

impl Default for DatabaseType {
    fn default() -> Self {
        Self::Sqlite
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type: "sqlite" or "postgres"
    #[serde(default)]
    pub db_type: DatabaseType,

    /// Connection string
    /// SQLite: "sqlite:pool.db" or "sqlite::memory:"
    /// PostgreSQL: "postgresql://user:pass@host/db"
    pub url: String,

    /// Auto-run migrations on startup
    #[serde(default = "default_auto_migrate")]
    pub auto_migrate: bool,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Idle connection timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
}

fn default_auto_migrate() -> bool {
    true
}

fn default_max_connections() -> u32 {
    10
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            url: "sqlite:pool.db".to_string(),
            auto_migrate: true,
            max_connections: 10,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl DatabaseConfig {
    /// Create a new SQLite configuration
    pub fn sqlite(path: &str) -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            url: format!("sqlite:{}", path),
            ..Default::default()
        }
    }

    /// Create an in-memory SQLite configuration (for testing)
    pub fn sqlite_memory() -> Self {
        // Use shared cache mode to allow multiple connections to same in-memory db
        Self {
            db_type: DatabaseType::Sqlite,
            url: format!("sqlite:file:test_{}?mode=memory&cache=shared", uuid::Uuid::new_v4()),
            ..Default::default()
        }
    }

    /// Create a PostgreSQL configuration
    pub fn postgres(url: &str) -> Self {
        Self {
            db_type: DatabaseType::Postgres,
            url: url.to_string(),
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> PoolResult<()> {
        if self.url.is_empty() {
            return Err(PoolError::validation("Database URL cannot be empty"));
        }

        match self.db_type {
            DatabaseType::Sqlite => {
                if !self.url.starts_with("sqlite:") {
                    return Err(PoolError::validation(
                        "SQLite URL must start with 'sqlite:'",
                    ));
                }
            }
            DatabaseType::Postgres => {
                if !self.url.starts_with("postgres://") && !self.url.starts_with("postgresql://") {
                    return Err(PoolError::validation(
                        "PostgreSQL URL must start with 'postgres://' or 'postgresql://'",
                    ));
                }
            }
        }

        if self.max_connections == 0 {
            return Err(PoolError::validation(
                "max_connections must be greater than 0",
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Database Health
// ============================================================================

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    /// Whether database is connected
    pub connected: bool,

    /// Whether migrations are up to date
    pub migrations_current: bool,

    /// Current schema version
    pub schema_version: i64,

    /// Number of active connections
    pub active_connections: u32,

    /// Number of idle connections
    pub idle_connections: u32,

    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,

    /// Error message if unhealthy
    pub error: Option<String>,
}

impl Default for DatabaseHealth {
    fn default() -> Self {
        Self {
            connected: false,
            migrations_current: false,
            schema_version: 0,
            active_connections: 0,
            idle_connections: 0,
            last_check: chrono::Utc::now(),
            error: None,
        }
    }
}

// ============================================================================
// Database Pool
// ============================================================================

/// Database connection pool
pub struct DatabasePool {
    pool: AnyPool,
    db_type: DatabaseType,
    config: DatabaseConfig,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(config: DatabaseConfig) -> PoolResult<Self> {
        config.validate()?;

        // Install SQLx drivers
        sqlx::any::install_default_drivers();

        let pool = AnyPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connect(&config.url)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to connect to database: {}", e)))?;

        Ok(Self {
            pool,
            db_type: config.db_type.clone(),
            config,
        })
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    /// Get database type
    pub fn db_type(&self) -> &DatabaseType {
        &self.db_type
    }

    /// Get configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> PoolResult<sqlx::pool::PoolConnection<sqlx::Any>> {
        self.pool
            .acquire()
            .await
            .map_err(|e| PoolError::internal(format!("Failed to acquire connection: {}", e)))
    }

    /// Execute a raw SQL query (for migrations)
    pub async fn execute(&self, sql: &str) -> PoolResult<()> {
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to execute query: {}", e)))?;
        Ok(())
    }

    /// Check database health
    pub async fn health_check(&self) -> DatabaseHealth {
        let mut health = DatabaseHealth {
            last_check: chrono::Utc::now(),
            ..Default::default()
        };

        // Check connection
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                health.connected = true;
            }
            Err(e) => {
                health.error = Some(format!("Connection failed: {}", e));
                return health;
            }
        }

        // Check migrations table and get version
        let version_result: Result<i64, _> = sqlx::query(
            "SELECT COALESCE(MAX(version), 0) as version FROM _sqlx_migrations",
        )
        .fetch_one(&self.pool)
        .await
        .and_then(|row| row.try_get("version"));

        match version_result {
            Ok(version) => {
                health.schema_version = version;
                health.migrations_current = true; // Will be validated by MigrationManager
            }
            Err(_) => {
                // Table might not exist yet
                health.schema_version = 0;
            }
        }

        // Pool stats
        health.active_connections = self.pool.size();
        health.idle_connections = self.pool.num_idle() as u32;

        health
    }

    /// Close all connections
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Check if pool is closed
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.db_type, DatabaseType::Sqlite);
        assert!(config.auto_migrate);
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_database_config_sqlite() {
        let config = DatabaseConfig::sqlite("test.db");
        assert_eq!(config.db_type, DatabaseType::Sqlite);
        assert_eq!(config.url, "sqlite:test.db");
    }

    #[test]
    fn test_database_config_sqlite_memory() {
        let config = DatabaseConfig::sqlite_memory();
        assert!(config.url.starts_with("sqlite:file:test_"));
        assert!(config.url.contains("mode=memory"));
    }

    #[test]
    fn test_database_config_postgres() {
        let config = DatabaseConfig::postgres("postgresql://user:pass@localhost/db");
        assert_eq!(config.db_type, DatabaseType::Postgres);
        assert!(config.url.starts_with("postgresql://"));
    }

    #[test]
    fn test_database_config_validation() {
        // Empty URL
        let config = DatabaseConfig {
            url: "".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid SQLite URL
        let config = DatabaseConfig {
            db_type: DatabaseType::Sqlite,
            url: "postgresql://localhost".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid Postgres URL
        let config = DatabaseConfig {
            db_type: DatabaseType::Postgres,
            url: "sqlite:test.db".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Zero connections
        let config = DatabaseConfig {
            max_connections: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Valid config
        let config = DatabaseConfig::default();
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_database_pool_sqlite_memory() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();

        // Check health
        let health = pool.health_check().await;
        assert!(health.connected);

        // Close pool
        pool.close().await;
        assert!(pool.is_closed());
    }

    #[tokio::test]
    async fn test_database_execute() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();

        // Create a table
        pool.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();

        // Insert data
        pool.execute("INSERT INTO test (id, name) VALUES (1, 'test')")
            .await
            .unwrap();

        // Verify
        let row = sqlx::query("SELECT name FROM test WHERE id = 1")
            .fetch_one(pool.pool())
            .await
            .unwrap();

        let name: String = row.try_get("name").unwrap();
        assert_eq!(name, "test");
    }
}

