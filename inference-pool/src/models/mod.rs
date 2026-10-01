//! Data models for the Inference Pool.
//!
//! This module contains:
//! - OpenAI-compatible request/response types
//! - Pool-specific types
//! - Orchestration event types

pub mod openai;
pub mod orchestration;
pub mod pool;

// Re-exports
pub use openai::*;
pub use orchestration::*;
pub use pool::*;







