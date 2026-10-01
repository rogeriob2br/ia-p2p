//! Billing data types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::SlaTier;

// ============================================================================
// Currency
// ============================================================================

/// Supported currencies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    /// Satoshis (1 BTC = 100,000,000 sats)
    Satoshis,
    /// US Dollars
    Usd,
    /// Euros
    Eur,
}

impl Default for Currency {
    fn default() -> Self {
        Self::Satoshis
    }
}

impl Currency {
    /// Get the currency symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::Satoshis => "sats",
            Currency::Usd => "$",
            Currency::Eur => "€",
        }
    }

    /// Get decimal places for display
    pub fn decimals(&self) -> u8 {
        match self {
            Currency::Satoshis => 0,
            Currency::Usd | Currency::Eur => 2,
        }
    }
}

// ============================================================================
// Billing Ticket
// ============================================================================

/// A billing ticket represents a single billable inference job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingTicket {
    /// Unique ticket ID
    pub ticket_id: String,

    /// Associated job ID
    pub job_id: String,

    /// Client ID
    pub client_id: String,

    /// Member who executed the job
    pub member_id: String,

    /// Model used
    pub model: String,

    /// Tokens consumed (prompt + completion)
    pub tokens_used: TokenUsage,

    /// Cost calculation
    pub cost: Cost,

    /// Client's SLA tier at time of job
    pub sla_tier: SlaTier,

    /// Ticket status
    pub status: TicketStatus,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Finalized timestamp (when payment confirmed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finalized_at: Option<DateTime<Utc>>,

    /// Associated invoice ID (when grouped into invoice)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<String>,
}

/// Token usage breakdown
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Prompt/input tokens
    pub prompt_tokens: u64,

    /// Completion/output tokens
    pub completion_tokens: u64,

    /// Total tokens (prompt + completion)
    pub total_tokens: u64,
}

impl TokenUsage {
    /// Create new token usage
    pub fn new(prompt_tokens: u64, completion_tokens: u64) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// Cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    /// Base amount (before tier multiplier)
    pub base_amount: f64,

    /// Tier multiplier applied
    pub tier_multiplier: f64,

    /// Final amount (after multiplier)
    pub final_amount: f64,

    /// Currency
    pub currency: Currency,

    /// Pool fee (percentage of final amount)
    pub pool_fee_percent: f64,

    /// Pool fee amount
    pub pool_fee_amount: f64,

    /// Member payment amount
    pub member_amount: f64,
}

impl Cost {
    /// Calculate cost from tokens
    pub fn calculate(
        tokens: &TokenUsage,
        rate_per_1k: f64,
        tier_multiplier: f64,
        pool_fee_percent: f64,
        currency: Currency,
    ) -> Self {
        let base_amount = (tokens.total_tokens as f64 / 1000.0) * rate_per_1k;
        let final_amount = base_amount * tier_multiplier;
        let pool_fee_amount = final_amount * (pool_fee_percent / 100.0);
        let member_amount = final_amount - pool_fee_amount;

        Self {
            base_amount,
            tier_multiplier,
            final_amount,
            currency,
            pool_fee_percent,
            pool_fee_amount,
            member_amount,
        }
    }
}

/// Ticket status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TicketStatus {
    /// Pending payment
    Pending,
    /// Added to invoice
    Invoiced,
    /// Payment confirmed
    Paid,
    /// Payment failed
    Failed,
    /// Cancelled/refunded
    Cancelled,
}

impl Default for TicketStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// ============================================================================
// Invoice
// ============================================================================

/// An invoice groups multiple billing tickets for a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique invoice ID
    pub invoice_id: String,

    /// Client ID
    pub client_id: String,

    /// Billing tickets included
    pub tickets: Vec<String>,

    /// Total amount
    pub total_amount: f64,

    /// Currency
    pub currency: Currency,

    /// Invoice status
    pub status: InvoiceStatus,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Due date
    pub due_at: DateTime<Utc>,

    /// Paid timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<DateTime<Utc>>,

    /// Payment reference (Lightning payment hash, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_reference: Option<String>,

    /// Usage summary
    pub usage_summary: UsageSummary,
}

/// Invoice status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    /// Draft, not yet sent
    Draft,
    /// Sent to client
    Pending,
    /// Payment received
    Paid,
    /// Overdue
    Overdue,
    /// Cancelled
    Cancelled,
}

impl Default for InvoiceStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Usage summary for an invoice
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageSummary {
    /// Total requests
    pub total_requests: u64,

    /// Total tokens
    pub total_tokens: u64,

    /// Breakdown by model
    pub by_model: HashMap<String, ModelUsage>,

    /// Breakdown by job type
    pub by_job_type: HashMap<String, u64>,
}

/// Usage per model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelUsage {
    /// Request count
    pub requests: u64,

    /// Token count
    pub tokens: u64,

    /// Cost
    pub cost: f64,
}

// ============================================================================
// Payment
// ============================================================================

/// A payment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Unique payment ID
    pub payment_id: String,

    /// Associated invoice ID
    pub invoice_id: String,

    /// Client ID
    pub client_id: String,

    /// Amount
    pub amount: f64,

    /// Currency
    pub currency: Currency,

    /// Payment method
    pub method: PaymentMethod,

    /// Payment status
    pub status: PaymentStatus,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Confirmed timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_at: Option<DateTime<Utc>>,

    /// Lightning payment hash (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_hash: Option<String>,

    /// Lightning payment preimage (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_preimage: Option<String>,

    /// External reference
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_reference: Option<String>,
}

/// Payment method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    /// Lightning Network
    Lightning,
    /// On-chain Bitcoin
    OnChain,
    /// Pre-paid credits
    Credits,
    /// External (Stripe, etc.)
    External,
}

impl Default for PaymentMethod {
    fn default() -> Self {
        Self::Lightning
    }
}

/// Payment status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    /// Pending confirmation
    Pending,
    /// Confirmed
    Confirmed,
    /// Failed
    Failed,
    /// Expired
    Expired,
    /// Refunded
    Refunded,
}

impl Default for PaymentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// ============================================================================
// Member Earnings
// ============================================================================

/// Earnings tracking for a pool member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberEarnings {
    /// Member ID
    pub member_id: String,

    /// Total earned (lifetime)
    pub total_earned: f64,

    /// Currency
    pub currency: Currency,

    /// Pending payout (not yet settled)
    pub pending_payout: f64,

    /// Last payout amount
    pub last_payout_amount: f64,

    /// Last payout timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_payout_at: Option<DateTime<Utc>>,

    /// Total jobs completed
    pub jobs_completed: u64,

    /// Total tokens processed
    pub tokens_processed: u64,

    /// Earnings by model
    pub by_model: HashMap<String, f64>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl MemberEarnings {
    /// Create new earnings record for a member
    pub fn new(member_id: String, currency: Currency) -> Self {
        let now = Utc::now();
        Self {
            member_id,
            total_earned: 0.0,
            currency,
            pending_payout: 0.0,
            last_payout_amount: 0.0,
            last_payout_at: None,
            jobs_completed: 0,
            tokens_processed: 0,
            by_model: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add earnings from a job
    pub fn add_earnings(&mut self, amount: f64, model: &str, tokens: u64) {
        self.total_earned += amount;
        self.pending_payout += amount;
        self.jobs_completed += 1;
        self.tokens_processed += tokens;
        *self.by_model.entry(model.to_string()).or_insert(0.0) += amount;
        self.updated_at = Utc::now();
    }

    /// Record a payout
    pub fn record_payout(&mut self, amount: f64) {
        self.pending_payout -= amount;
        self.last_payout_amount = amount;
        self.last_payout_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_symbol() {
        assert_eq!(Currency::Satoshis.symbol(), "sats");
        assert_eq!(Currency::Usd.symbol(), "$");
        assert_eq!(Currency::Eur.symbol(), "€");
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_cost_calculation() {
        let usage = TokenUsage::new(1000, 500); // 1500 total
        let cost = Cost::calculate(&usage, 10.0, 1.0, 10.0, Currency::Satoshis);

        // 1500/1000 * 10 = 15.0 base
        assert!((cost.base_amount - 15.0).abs() < 0.001);
        assert!((cost.final_amount - 15.0).abs() < 0.001);
        // 15 * 0.10 = 1.5 pool fee
        assert!((cost.pool_fee_amount - 1.5).abs() < 0.001);
        // 15 - 1.5 = 13.5 member amount
        assert!((cost.member_amount - 13.5).abs() < 0.001);
    }

    #[test]
    fn test_cost_with_tier_multiplier() {
        let usage = TokenUsage::new(1000, 0); // 1000 total
        let cost = Cost::calculate(&usage, 10.0, 0.8, 10.0, Currency::Usd);

        // 1000/1000 * 10 = 10.0 base
        assert!((cost.base_amount - 10.0).abs() < 0.001);
        // 10 * 0.8 = 8.0 final
        assert!((cost.final_amount - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_member_earnings() {
        let mut earnings = MemberEarnings::new("member-1".to_string(), Currency::Satoshis);

        earnings.add_earnings(100.0, "llama-7b", 1000);
        assert_eq!(earnings.total_earned, 100.0);
        assert_eq!(earnings.pending_payout, 100.0);
        assert_eq!(earnings.jobs_completed, 1);
        assert_eq!(earnings.tokens_processed, 1000);

        earnings.add_earnings(50.0, "llama-7b", 500);
        assert_eq!(earnings.total_earned, 150.0);
        assert_eq!(earnings.by_model.get("llama-7b"), Some(&150.0));

        earnings.record_payout(100.0);
        assert_eq!(earnings.pending_payout, 50.0);
        assert_eq!(earnings.last_payout_amount, 100.0);
        assert!(earnings.last_payout_at.is_some());
    }

    #[test]
    fn test_ticket_status_serialization() {
        assert_eq!(
            serde_json::to_string(&TicketStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&TicketStatus::Paid).unwrap(),
            "\"paid\""
        );
    }

    #[test]
    fn test_invoice_status_serialization() {
        assert_eq!(
            serde_json::to_string(&InvoiceStatus::Draft).unwrap(),
            "\"draft\""
        );
        assert_eq!(
            serde_json::to_string(&InvoiceStatus::Paid).unwrap(),
            "\"paid\""
        );
    }

    #[test]
    fn test_payment_method_serialization() {
        assert_eq!(
            serde_json::to_string(&PaymentMethod::Lightning).unwrap(),
            "\"lightning\""
        );
        assert_eq!(
            serde_json::to_string(&PaymentMethod::OnChain).unwrap(),
            "\"on_chain\""
        );
    }
}






