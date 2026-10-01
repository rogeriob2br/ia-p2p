//! Lightning Channel Manager module.
//!
//! Manages direct Lightning channels with Pool_Members.
//!
//! ## Features
//!
//! - Channel opening on registration
//! - Instant payments via direct channel
//! - Liquidity monitoring and rebalancing
//! - Cooperative channel closure
//! - Fallback to Lightning routing
//!
//! ## Backends
//!
//! - **LND**: Lightning Network Daemon (REST API)
//! - **CLN**: Core Lightning (Unix socket JSON-RPC)
//! - **Mock**: In-memory for testing

mod manager;
mod types;

// Lightning backends
pub mod cln;
pub mod lnd;

pub use manager::{ChannelManager, InMemoryChannelManager, LightningBackend};
pub use types::{
    Channel, ChannelConfig, ChannelEvent, ChannelState, ChannelStats, PaymentRequest,
    PaymentResult, PaymentState,
};

// Re-export backend-specific types
pub use cln::{ClnChannelManager, ClnClient, ClnConfig};
pub use lnd::{LndChannelManager, LndClient, LndConfig};
