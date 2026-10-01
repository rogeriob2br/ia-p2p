//! Billing System module.
//!
//! Complete billing for clients and Pool_Members.
//!
//! ## Features
//!
//! - Billing tickets per inference
//! - Invoice generation
//! - Multi-currency support (satoshis, USD)
//! - Lightning Network payments
//! - Member earnings tracking

mod service;
mod types;

pub use service::{BillingConfig, BillingService, InMemoryBillingService};
pub use types::{
    BillingTicket, Cost, Currency, Invoice, InvoiceStatus, MemberEarnings, ModelUsage, Payment,
    PaymentMethod, PaymentStatus, TicketStatus, TokenUsage, UsageSummary,
};
