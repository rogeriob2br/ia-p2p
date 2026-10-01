//! CLN (Core Lightning) integration.
//!
//! Provides client for interacting with Core Lightning nodes.
//!
//! ## Connection Methods
//!
//! CLN supports multiple interfaces:
//! - Unix socket (JSON-RPC) - Default, most common
//! - gRPC plugin (cln-grpc) - For remote connections
//!
//! ## Example
//!
//! ```ignore
//! use inference_pool::channel_manager::cln::{ClnClient, ClnConfig};
//!
//! let config = ClnConfig {
//!     socket_path: "/path/to/.lightning/lightning-rpc".to_string(),
//! };
//!
//! let client = ClnClient::connect(config).await?;
//! let info = client.get_info().await?;
//! ```

mod client;
mod types;

pub use client::{ClnChannelManager, ClnClient, ClnConfig};
pub use types::*;






