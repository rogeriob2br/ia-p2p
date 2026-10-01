//! p2p-core: Core P2P networking for decentralized AI network
//!
//! Provides gossipsub messaging, mDNS discovery, and request-response protocols.

pub mod messages;
pub mod network;

pub use messages::*;
pub use network::*;












