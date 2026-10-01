//! LND (Lightning Network Daemon) integration.
//!
//! Provides gRPC client for interacting with LND nodes.
//!
//! ## Authentication
//!
//! LND requires:
//! - TLS certificate for secure connection
//! - Macaroon for authentication/authorization
//!
//! ## Example
//!
//! ```ignore
//! use inference_pool::channel_manager::lnd::{LndClient, LndConfig};
//!
//! let config = LndConfig {
//!     endpoint: "https://localhost:10009".to_string(),
//!     cert_path: "/path/to/tls.cert".to_string(),
//!     macaroon_path: "/path/to/admin.macaroon".to_string(),
//! };
//!
//! let client = LndClient::connect(config).await?;
//! let info = client.get_info().await?;
//! ```

mod client;
mod types;

pub use client::{LndChannelManager, LndClient, LndConfig};
pub use types::*;






