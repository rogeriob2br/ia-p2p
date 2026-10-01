//! Database Layer for Inference Pool.
//!
//! Provides persistence for critical data:
//! - Pool members (registry)
//! - API keys (authentication)
//! - Billing tickets and invoices
//! - Lightning channels
//!
//! ## Supported Databases
//!
//! - **SQLite**: For development and testing
//! - **PostgreSQL**: For production
//!
//! ## Example
//!
//! ```ignore
//! use inference_pool::database::{DatabaseConfig, DatabasePool, DatabaseType};
//!
//! let config = DatabaseConfig {
//!     db_type: DatabaseType::Sqlite,
//!     url: "sqlite:pool.db".to_string(),
//!     auto_migrate: true,
//!     max_connections: 5,
//! };
//!
//! let pool = DatabasePool::new(config).await?;
//! pool.migrate().await?;
//! ```

mod pool;
mod migrations;

pub mod queries;

pub use pool::{DatabaseConfig, DatabaseHealth, DatabasePool, DatabaseType};
pub use migrations::{Migration, MigrationManager, MigrationRecord};


