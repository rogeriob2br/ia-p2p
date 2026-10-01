//! Database migration management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::pool::DatabasePool;
use crate::error::{PoolError, PoolResult};

// ============================================================================
// Migration Types
// ============================================================================

/// A database migration
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration version (must be unique and sequential)
    pub version: i64,

    /// Migration name (descriptive)
    pub name: String,

    /// SQL to apply the migration
    pub up_sql: String,

    /// SQL to revert the migration
    pub down_sql: String,
}

/// Record of an applied migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration version
    pub version: i64,

    /// Migration name
    pub name: String,

    /// When the migration was applied
    pub applied_at: DateTime<Utc>,

    /// Whether migration succeeded
    pub success: bool,

    /// Checksum of the migration SQL
    pub checksum: String,
}

// ============================================================================
// Built-in Migrations
// ============================================================================

/// Get all built-in migrations
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "initial_schema".to_string(),
            up_sql: MIGRATION_001_UP.to_string(),
            down_sql: MIGRATION_001_DOWN.to_string(),
        },
    ]
}

const MIGRATION_001_UP: &str = r#"
-- Pool Members (Registry)
CREATE TABLE IF NOT EXISTS pool_members (
    node_id TEXT PRIMARY KEY,
    peer_id TEXT NOT NULL,
    capabilities TEXT NOT NULL,  -- JSON
    models TEXT NOT NULL,        -- JSON array
    status TEXT NOT NULL DEFAULT 'inactive',
    lightning_pubkey TEXT,
    multiaddrs TEXT,             -- JSON array
    registered_at TEXT NOT NULL,
    last_heartbeat TEXT,
    metrics TEXT                 -- JSON
);

CREATE INDEX IF NOT EXISTS idx_pool_members_status ON pool_members(status);
CREATE INDEX IF NOT EXISTS idx_pool_members_lightning_pubkey ON pool_members(lightning_pubkey);

-- API Keys
CREATE TABLE IF NOT EXISTS api_keys (
    key_id TEXT PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,
    client_id TEXT NOT NULL,
    tier TEXT NOT NULL DEFAULT 'standard',
    rate_limit TEXT NOT NULL,    -- JSON
    created_at TEXT NOT NULL,
    expires_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT,
    usage_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_client_id ON api_keys(client_id);

-- Billing Tickets
CREATE TABLE IF NOT EXISTS billing_tickets (
    ticket_id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    model TEXT NOT NULL,
    tokens_input INTEGER NOT NULL,
    tokens_output INTEGER NOT NULL,
    client_cost_amount REAL NOT NULL,
    client_cost_currency TEXT NOT NULL DEFAULT 'USD',
    member_payment_amount REAL NOT NULL,
    member_payment_currency TEXT NOT NULL DEFAULT 'BTC',
    pool_fee_amount REAL NOT NULL,
    pool_fee_currency TEXT NOT NULL DEFAULT 'BTC',
    payment_status TEXT NOT NULL DEFAULT 'pending',
    channel_id TEXT,
    payment_hash TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_billing_tickets_client_id ON billing_tickets(client_id);
CREATE INDEX IF NOT EXISTS idx_billing_tickets_member_id ON billing_tickets(member_id);
CREATE INDEX IF NOT EXISTS idx_billing_tickets_payment_status ON billing_tickets(payment_status);
CREATE INDEX IF NOT EXISTS idx_billing_tickets_timestamp ON billing_tickets(timestamp);

-- Invoices
CREATE TABLE IF NOT EXISTS invoices (
    invoice_id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    ticket_ids TEXT NOT NULL,    -- JSON array
    subtotal_amount REAL NOT NULL,
    subtotal_currency TEXT NOT NULL DEFAULT 'USD',
    tax_amount REAL,
    tax_currency TEXT,
    total_amount REAL NOT NULL,
    total_currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'pending',
    due_date TEXT NOT NULL,
    paid_at TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_invoices_client_id ON invoices(client_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);

-- Lightning Channels
CREATE TABLE IF NOT EXISTS lightning_channels (
    channel_id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    node_pubkey TEXT NOT NULL,
    capacity_sats INTEGER NOT NULL,
    local_balance_sats INTEGER NOT NULL,
    remote_balance_sats INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    private_channel INTEGER NOT NULL DEFAULT 0,
    opened_at TEXT NOT NULL,
    last_payment_at TEXT,
    total_payments INTEGER DEFAULT 0,
    total_sats_sent INTEGER DEFAULT 0,
    total_sats_received INTEGER DEFAULT 0,
    funding_txid TEXT,
    funding_output INTEGER
);

CREATE INDEX IF NOT EXISTS idx_lightning_channels_member_id ON lightning_channels(member_id);
CREATE INDEX IF NOT EXISTS idx_lightning_channels_status ON lightning_channels(status);
CREATE INDEX IF NOT EXISTS idx_lightning_channels_node_pubkey ON lightning_channels(node_pubkey);

-- Member Payouts
CREATE TABLE IF NOT EXISTS member_payouts (
    payout_id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    ticket_ids TEXT NOT NULL,    -- JSON array
    gross_amount REAL NOT NULL,
    pool_fee_amount REAL NOT NULL,
    net_amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'BTC',
    status TEXT NOT NULL DEFAULT 'pending',
    payment_hash TEXT,
    paid_at TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_member_payouts_member_id ON member_payouts(member_id);
CREATE INDEX IF NOT EXISTS idx_member_payouts_status ON member_payouts(status);
"#;

const MIGRATION_001_DOWN: &str = r#"
DROP TABLE IF EXISTS member_payouts;
DROP TABLE IF EXISTS lightning_channels;
DROP TABLE IF EXISTS invoices;
DROP TABLE IF EXISTS billing_tickets;
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS pool_members;
"#;

// ============================================================================
// Migration Manager
// ============================================================================

/// Manages database migrations
pub struct MigrationManager {
    /// The database pool
    pub pool: DatabasePool,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            migrations: get_migrations(),
        }
    }

    /// Create a new migration manager with custom migrations
    pub fn with_migrations(pool: DatabasePool, migrations: Vec<Migration>) -> Self {
        Self { pool, migrations }
    }

    /// Ensure migrations table exists
    async fn ensure_migrations_table(&self) -> PoolResult<()> {
        self.pool
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS _pool_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at TEXT NOT NULL,
                    success INTEGER NOT NULL DEFAULT 1,
                    checksum TEXT NOT NULL
                )
                "#,
            )
            .await
    }

    /// Get current schema version
    pub async fn current_version(&self) -> PoolResult<i64> {
        self.ensure_migrations_table().await?;

        let result = sqlx::query("SELECT COALESCE(MAX(version), 0) as version FROM _pool_migrations")
            .fetch_one(self.pool.pool())
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get schema version: {}", e)))?;

        let version: i64 = result.try_get("version").unwrap_or(0);
        Ok(version)
    }

    /// Get pending migrations
    pub async fn pending_migrations(&self) -> PoolResult<Vec<&Migration>> {
        let current = self.current_version().await?;

        let pending: Vec<&Migration> = self
            .migrations
            .iter()
            .filter(|m| m.version > current)
            .collect();

        Ok(pending)
    }

    /// Run all pending migrations
    pub async fn migrate_up(&self) -> PoolResult<Vec<MigrationRecord>> {
        self.ensure_migrations_table().await?;

        let pending = self.pending_migrations().await?;
        let mut records = Vec::new();

        for migration in pending {
            tracing::info!(
                version = migration.version,
                name = %migration.name,
                "Running migration"
            );

            let checksum = format!("{:x}", md5::compute(&migration.up_sql));
            let applied_at = Utc::now();

            // Execute migration
            match self.pool.execute(&migration.up_sql).await {
                Ok(_) => {
                    // Record success
                    let record = MigrationRecord {
                        version: migration.version,
                        name: migration.name.clone(),
                        applied_at,
                        success: true,
                        checksum: checksum.clone(),
                    };

                    sqlx::query(
                        "INSERT INTO _pool_migrations (version, name, applied_at, success, checksum) VALUES (?, ?, ?, 1, ?)"
                    )
                    .bind(migration.version)
                    .bind(&migration.name)
                    .bind(applied_at.to_rfc3339())
                    .bind(&checksum)
                    .execute(self.pool.pool())
                    .await
                    .map_err(|e| PoolError::internal(format!("Failed to record migration: {}", e)))?;

                    tracing::info!(
                        version = migration.version,
                        name = %migration.name,
                        "Migration completed successfully"
                    );

                    records.push(record);
                }
                Err(e) => {
                    tracing::error!(
                        version = migration.version,
                        name = %migration.name,
                        error = %e,
                        "Migration failed"
                    );
                    return Err(PoolError::internal(format!(
                        "Migration {} ({}) failed: {}",
                        migration.version, migration.name, e
                    )));
                }
            }
        }

        Ok(records)
    }

    /// Rollback the last migration
    pub async fn migrate_down(&self) -> PoolResult<Option<MigrationRecord>> {
        let current = self.current_version().await?;

        if current == 0 {
            return Ok(None);
        }

        let migration = self
            .migrations
            .iter()
            .find(|m| m.version == current)
            .ok_or_else(|| {
                PoolError::internal(format!("Migration {} not found for rollback", current))
            })?;

        tracing::info!(
            version = migration.version,
            name = %migration.name,
            "Rolling back migration"
        );

        // Execute rollback
        self.pool.execute(&migration.down_sql).await?;

        // Remove from migrations table
        sqlx::query("DELETE FROM _pool_migrations WHERE version = ?")
            .bind(migration.version)
            .execute(self.pool.pool())
            .await
            .map_err(|e| PoolError::internal(format!("Failed to remove migration record: {}", e)))?;

        tracing::info!(
            version = migration.version,
            name = %migration.name,
            "Migration rolled back successfully"
        );

        Ok(Some(MigrationRecord {
            version: migration.version,
            name: migration.name.clone(),
            applied_at: Utc::now(),
            success: true,
            checksum: String::new(),
        }))
    }

    /// Get migration history
    pub async fn migration_history(&self) -> PoolResult<Vec<MigrationRecord>> {
        self.ensure_migrations_table().await?;

        let rows = sqlx::query(
            "SELECT version, name, applied_at, success, checksum FROM _pool_migrations ORDER BY version"
        )
        .fetch_all(self.pool.pool())
        .await
        .map_err(|e| PoolError::internal(format!("Failed to get migration history: {}", e)))?;

        let records: Vec<MigrationRecord> = rows
            .iter()
            .map(|row| {
                let applied_at_str: String = row.try_get("applied_at").unwrap_or_default();
                let applied_at = DateTime::parse_from_rfc3339(&applied_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                MigrationRecord {
                    version: row.try_get("version").unwrap_or(0),
                    name: row.try_get("name").unwrap_or_default(),
                    applied_at,
                    success: row.try_get::<i32, _>("success").unwrap_or(0) == 1,
                    checksum: row.try_get("checksum").unwrap_or_default(),
                }
            })
            .collect();

        Ok(records)
    }

    /// Check if all migrations are applied
    pub async fn is_current(&self) -> PoolResult<bool> {
        let pending = self.pending_migrations().await?;
        Ok(pending.is_empty())
    }
}

// Simple MD5 for checksums
mod md5 {
    pub struct Digest([u8; 16]);

    impl std::fmt::LowerHex for Digest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for byte in &self.0 {
                write!(f, "{:02x}", byte)?;
            }
            Ok(())
        }
    }

    pub fn compute(data: &str) -> Digest {
        use sha2::{Digest as Sha2Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[..16]);
        Digest(bytes)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;

    #[tokio::test]
    async fn test_migration_manager_creation() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        assert!(!manager.migrations.is_empty());
    }

    #[tokio::test]
    async fn test_migrate_up() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        // Run migrations
        let records = manager.migrate_up().await.unwrap();
        assert!(!records.is_empty());

        // Check version
        let version = manager.current_version().await.unwrap();
        assert_eq!(version, 1);

        // Check no pending
        let pending = manager.pending_migrations().await.unwrap();
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_migrate_up_idempotent() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        // Run twice
        manager.migrate_up().await.unwrap();
        let records = manager.migrate_up().await.unwrap();

        // Second run should have no records (all already applied)
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn test_migrate_down() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        // Run up
        manager.migrate_up().await.unwrap();
        assert_eq!(manager.current_version().await.unwrap(), 1);

        // Run down
        let record = manager.migrate_down().await.unwrap();
        assert!(record.is_some());
        assert_eq!(manager.current_version().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_migration_history() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        // Run migrations
        manager.migrate_up().await.unwrap();

        // Get history
        let history = manager.migration_history().await.unwrap();
        assert!(!history.is_empty());
        assert!(history[0].success);
    }

    #[tokio::test]
    async fn test_tables_created() {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);

        // Run migrations
        manager.migrate_up().await.unwrap();

        // Check tables exist by trying to query them
        let db_pool = manager.pool.pool();

        // pool_members
        sqlx::query("SELECT COUNT(*) FROM pool_members")
            .fetch_one(db_pool)
            .await
            .unwrap();

        // api_keys
        sqlx::query("SELECT COUNT(*) FROM api_keys")
            .fetch_one(db_pool)
            .await
            .unwrap();

        // billing_tickets
        sqlx::query("SELECT COUNT(*) FROM billing_tickets")
            .fetch_one(db_pool)
            .await
            .unwrap();

        // invoices
        sqlx::query("SELECT COUNT(*) FROM invoices")
            .fetch_one(db_pool)
            .await
            .unwrap();

        // lightning_channels
        sqlx::query("SELECT COUNT(*) FROM lightning_channels")
            .fetch_one(db_pool)
            .await
            .unwrap();
    }
}

