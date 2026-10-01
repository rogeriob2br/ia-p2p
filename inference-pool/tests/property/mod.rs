//! Property-based tests for the Inference Pool.
//!
//! Each test references a property from the design document.
//! Format: `Feature: api-integration, Property N: <property_text>`
//!
//! Minimum 100 iterations per property test.

pub mod api_properties;
pub mod auth_properties;
pub mod billing_properties;
pub mod error_properties;
pub mod heartbeat_properties;
pub mod openai_properties;
pub mod registry_properties;
pub mod router_properties;

