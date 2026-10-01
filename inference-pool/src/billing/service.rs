//! Billing service implementation.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{PoolError, PoolResult};
use crate::models::SlaTier;

use super::types::{
    BillingTicket, Cost, Currency, Invoice, InvoiceStatus, MemberEarnings, ModelUsage, Payment,
    PaymentMethod, PaymentStatus, TicketStatus, TokenUsage, UsageSummary,
};

// ============================================================================
// Billing Service Trait
// ============================================================================

/// Billing service trait
#[async_trait]
pub trait BillingService: Send + Sync {
    // Tickets
    /// Create a billing ticket for a completed job
    async fn create_ticket(
        &self,
        job_id: &str,
        client_id: &str,
        member_id: &str,
        model: &str,
        tokens: TokenUsage,
        sla_tier: SlaTier,
    ) -> PoolResult<BillingTicket>;

    /// Get a ticket by ID
    async fn get_ticket(&self, ticket_id: &str) -> PoolResult<BillingTicket>;

    /// Get tickets for a client
    async fn get_client_tickets(&self, client_id: &str) -> PoolResult<Vec<BillingTicket>>;

    /// Get pending tickets for a client
    async fn get_pending_tickets(&self, client_id: &str) -> PoolResult<Vec<BillingTicket>>;

    // Invoices
    /// Create an invoice from pending tickets
    async fn create_invoice(&self, client_id: &str) -> PoolResult<Invoice>;

    /// Get an invoice by ID
    async fn get_invoice(&self, invoice_id: &str) -> PoolResult<Invoice>;

    /// Get invoices for a client
    async fn get_client_invoices(&self, client_id: &str) -> PoolResult<Vec<Invoice>>;

    /// Mark invoice as paid
    async fn mark_invoice_paid(
        &self,
        invoice_id: &str,
        payment_reference: &str,
    ) -> PoolResult<Invoice>;

    // Payments
    /// Record a payment
    async fn record_payment(
        &self,
        invoice_id: &str,
        amount: f64,
        method: PaymentMethod,
        payment_hash: Option<String>,
    ) -> PoolResult<Payment>;

    /// Get payment by ID
    async fn get_payment(&self, payment_id: &str) -> PoolResult<Payment>;

    /// Confirm a payment
    async fn confirm_payment(
        &self,
        payment_id: &str,
        preimage: Option<String>,
    ) -> PoolResult<Payment>;

    // Member Earnings
    /// Get earnings for a member
    async fn get_member_earnings(&self, member_id: &str) -> PoolResult<MemberEarnings>;

    /// Record payout to a member
    async fn record_member_payout(&self, member_id: &str, amount: f64) -> PoolResult<()>;

    // Configuration
    /// Get rate per 1k tokens for a model
    fn get_rate_per_1k(&self, model: &str) -> f64;

    /// Get tier multiplier
    fn get_tier_multiplier(&self, tier: SlaTier) -> f64;

    /// Get pool fee percentage
    fn get_pool_fee_percent(&self) -> f64;

    /// Get default currency
    fn get_currency(&self) -> Currency;
}

// ============================================================================
// Billing Configuration
// ============================================================================

/// Billing configuration
#[derive(Debug, Clone)]
pub struct BillingConfig {
    /// Base rate per 1k tokens
    pub base_rate_per_1k: f64,

    /// Pool fee percentage (0-100)
    pub pool_fee_percent: f64,

    /// Default currency
    pub currency: Currency,

    /// Tier multipliers
    pub tier_multipliers: std::collections::HashMap<SlaTier, f64>,

    /// Model-specific rates (if different from base)
    pub model_rates: std::collections::HashMap<String, f64>,

    /// Invoice due days
    pub invoice_due_days: i64,

    /// Minimum invoice amount
    pub minimum_invoice_amount: f64,
}

impl Default for BillingConfig {
    fn default() -> Self {
        use std::collections::HashMap;

        let mut tier_multipliers = HashMap::new();
        tier_multipliers.insert(SlaTier::Free, 1.0);
        tier_multipliers.insert(SlaTier::Basic, 1.0);
        tier_multipliers.insert(SlaTier::Pro, 0.9);
        tier_multipliers.insert(SlaTier::Enterprise, 0.8);

        Self {
            base_rate_per_1k: 10.0, // 10 satoshis per 1k tokens
            pool_fee_percent: 10.0,
            currency: Currency::Satoshis,
            tier_multipliers,
            model_rates: HashMap::new(),
            invoice_due_days: 7,
            minimum_invoice_amount: 100.0,
        }
    }
}

// ============================================================================
// In-Memory Implementation
// ============================================================================

/// In-memory billing service implementation
pub struct InMemoryBillingService {
    config: BillingConfig,
    tickets: DashMap<String, BillingTicket>,
    invoices: DashMap<String, Invoice>,
    payments: DashMap<String, Payment>,
    member_earnings: DashMap<String, MemberEarnings>,
    client_tickets: DashMap<String, Vec<String>>,
    client_invoices: DashMap<String, Vec<String>>,
    ticket_counter: Arc<RwLock<u64>>,
    invoice_counter: Arc<RwLock<u64>>,
    payment_counter: Arc<RwLock<u64>>,
}

impl InMemoryBillingService {
    /// Create a new in-memory billing service
    pub fn new(config: BillingConfig) -> Self {
        Self {
            config,
            tickets: DashMap::new(),
            invoices: DashMap::new(),
            payments: DashMap::new(),
            member_earnings: DashMap::new(),
            client_tickets: DashMap::new(),
            client_invoices: DashMap::new(),
            ticket_counter: Arc::new(RwLock::new(0)),
            invoice_counter: Arc::new(RwLock::new(0)),
            payment_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Generate ticket ID
    async fn next_ticket_id(&self) -> String {
        let mut counter = self.ticket_counter.write().await;
        *counter += 1;
        format!("ticket-{:08}", *counter)
    }

    /// Generate invoice ID
    async fn next_invoice_id(&self) -> String {
        let mut counter = self.invoice_counter.write().await;
        *counter += 1;
        format!("inv-{:08}", *counter)
    }

    /// Generate payment ID
    async fn next_payment_id(&self) -> String {
        let mut counter = self.payment_counter.write().await;
        *counter += 1;
        format!("pay-{:08}", *counter)
    }

    /// Ensure member earnings record exists
    fn ensure_member_earnings(&self, member_id: &str) {
        if !self.member_earnings.contains_key(member_id) {
            self.member_earnings.insert(
                member_id.to_string(),
                MemberEarnings::new(member_id.to_string(), self.config.currency),
            );
        }
    }
}

#[async_trait]
impl BillingService for InMemoryBillingService {
    async fn create_ticket(
        &self,
        job_id: &str,
        client_id: &str,
        member_id: &str,
        model: &str,
        tokens: TokenUsage,
        sla_tier: SlaTier,
    ) -> PoolResult<BillingTicket> {
        let ticket_id = self.next_ticket_id().await;
        let rate = self.get_rate_per_1k(model);
        let multiplier = self.get_tier_multiplier(sla_tier);
        let cost = Cost::calculate(
            &tokens,
            rate,
            multiplier,
            self.config.pool_fee_percent,
            self.config.currency,
        );

        let ticket = BillingTicket {
            ticket_id: ticket_id.clone(),
            job_id: job_id.to_string(),
            client_id: client_id.to_string(),
            member_id: member_id.to_string(),
            model: model.to_string(),
            tokens_used: tokens,
            cost: cost.clone(),
            sla_tier,
            status: TicketStatus::Pending,
            created_at: Utc::now(),
            finalized_at: None,
            invoice_id: None,
        };

        // Store ticket
        self.tickets.insert(ticket_id.clone(), ticket.clone());

        // Add to client's tickets
        self.client_tickets
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(ticket_id);

        // Update member earnings
        self.ensure_member_earnings(member_id);
        if let Some(mut earnings) = self.member_earnings.get_mut(member_id) {
            earnings.add_earnings(cost.member_amount, model, tokens.total_tokens);
        }

        Ok(ticket)
    }

    async fn get_ticket(&self, ticket_id: &str) -> PoolResult<BillingTicket> {
        self.tickets
            .get(ticket_id)
            .map(|t| t.clone())
            .ok_or_else(|| PoolError::not_found(format!("Ticket not found: {}", ticket_id)))
    }

    async fn get_client_tickets(&self, client_id: &str) -> PoolResult<Vec<BillingTicket>> {
        let ticket_ids = self
            .client_tickets
            .get(client_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        let tickets: Vec<BillingTicket> = ticket_ids
            .iter()
            .filter_map(|id| self.tickets.get(id).map(|t| t.clone()))
            .collect();

        Ok(tickets)
    }

    async fn get_pending_tickets(&self, client_id: &str) -> PoolResult<Vec<BillingTicket>> {
        let tickets = self.get_client_tickets(client_id).await?;
        Ok(tickets
            .into_iter()
            .filter(|t| t.status == TicketStatus::Pending)
            .collect())
    }

    async fn create_invoice(&self, client_id: &str) -> PoolResult<Invoice> {
        let pending_tickets = self.get_pending_tickets(client_id).await?;

        if pending_tickets.is_empty() {
            return Err(PoolError::validation("No pending tickets to invoice"));
        }

        let total_amount: f64 = pending_tickets.iter().map(|t| t.cost.final_amount).sum();

        if total_amount < self.config.minimum_invoice_amount {
            return Err(PoolError::validation(format!(
                "Total amount {} is below minimum {} {}",
                total_amount,
                self.config.minimum_invoice_amount,
                self.config.currency.symbol()
            )));
        }

        let invoice_id = self.next_invoice_id().await;
        let now = Utc::now();

        // Build usage summary
        let mut usage_summary = UsageSummary::default();
        let mut ticket_ids = Vec::new();

        for ticket in &pending_tickets {
            ticket_ids.push(ticket.ticket_id.clone());
            usage_summary.total_requests += 1;
            usage_summary.total_tokens += ticket.tokens_used.total_tokens;

            let model_usage = usage_summary
                .by_model
                .entry(ticket.model.clone())
                .or_insert_with(ModelUsage::default);
            model_usage.requests += 1;
            model_usage.tokens += ticket.tokens_used.total_tokens;
            model_usage.cost += ticket.cost.final_amount;

            let job_type = "chat_completion".to_string(); // Default for now
            *usage_summary.by_job_type.entry(job_type).or_insert(0) += 1;
        }

        let invoice = Invoice {
            invoice_id: invoice_id.clone(),
            client_id: client_id.to_string(),
            tickets: ticket_ids.clone(),
            total_amount,
            currency: self.config.currency,
            status: InvoiceStatus::Pending,
            created_at: now,
            due_at: now + Duration::days(self.config.invoice_due_days),
            paid_at: None,
            payment_reference: None,
            usage_summary,
        };

        // Update ticket statuses
        for ticket_id in &ticket_ids {
            if let Some(mut ticket) = self.tickets.get_mut(ticket_id) {
                ticket.status = TicketStatus::Invoiced;
                ticket.invoice_id = Some(invoice_id.clone());
            }
        }

        // Store invoice
        self.invoices.insert(invoice_id.clone(), invoice.clone());

        // Add to client's invoices
        self.client_invoices
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(invoice_id);

        Ok(invoice)
    }

    async fn get_invoice(&self, invoice_id: &str) -> PoolResult<Invoice> {
        self.invoices
            .get(invoice_id)
            .map(|i| i.clone())
            .ok_or_else(|| PoolError::not_found(format!("Invoice not found: {}", invoice_id)))
    }

    async fn get_client_invoices(&self, client_id: &str) -> PoolResult<Vec<Invoice>> {
        let invoice_ids = self
            .client_invoices
            .get(client_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        let invoices: Vec<Invoice> = invoice_ids
            .iter()
            .filter_map(|id| self.invoices.get(id).map(|i| i.clone()))
            .collect();

        Ok(invoices)
    }

    async fn mark_invoice_paid(
        &self,
        invoice_id: &str,
        payment_reference: &str,
    ) -> PoolResult<Invoice> {
        let mut invoice = self.get_invoice(invoice_id).await?;

        if invoice.status == InvoiceStatus::Paid {
            return Err(PoolError::validation("Invoice already paid"));
        }

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(Utc::now());
        invoice.payment_reference = Some(payment_reference.to_string());

        // Update ticket statuses
        for ticket_id in &invoice.tickets {
            if let Some(mut ticket) = self.tickets.get_mut(ticket_id) {
                ticket.status = TicketStatus::Paid;
                ticket.finalized_at = Some(Utc::now());
            }
        }

        self.invoices.insert(invoice_id.to_string(), invoice.clone());

        Ok(invoice)
    }

    async fn record_payment(
        &self,
        invoice_id: &str,
        amount: f64,
        method: PaymentMethod,
        payment_hash: Option<String>,
    ) -> PoolResult<Payment> {
        let invoice = self.get_invoice(invoice_id).await?;

        if (invoice.total_amount - amount).abs() > 0.001 {
            return Err(PoolError::validation(format!(
                "Payment amount {} doesn't match invoice total {}",
                amount, invoice.total_amount
            )));
        }

        let payment_id = self.next_payment_id().await;

        let payment = Payment {
            payment_id: payment_id.clone(),
            invoice_id: invoice_id.to_string(),
            client_id: invoice.client_id.clone(),
            amount,
            currency: invoice.currency,
            method,
            status: PaymentStatus::Pending,
            created_at: Utc::now(),
            confirmed_at: None,
            payment_hash,
            payment_preimage: None,
            external_reference: None,
        };

        self.payments.insert(payment_id, payment.clone());

        Ok(payment)
    }

    async fn get_payment(&self, payment_id: &str) -> PoolResult<Payment> {
        self.payments
            .get(payment_id)
            .map(|p| p.clone())
            .ok_or_else(|| PoolError::not_found(format!("Payment not found: {}", payment_id)))
    }

    async fn confirm_payment(
        &self,
        payment_id: &str,
        preimage: Option<String>,
    ) -> PoolResult<Payment> {
        let mut payment = self.get_payment(payment_id).await?;

        if payment.status == PaymentStatus::Confirmed {
            return Err(PoolError::validation("Payment already confirmed"));
        }

        payment.status = PaymentStatus::Confirmed;
        payment.confirmed_at = Some(Utc::now());
        payment.payment_preimage = preimage;

        self.payments
            .insert(payment_id.to_string(), payment.clone());

        // Mark invoice as paid
        self.mark_invoice_paid(&payment.invoice_id, payment_id)
            .await?;

        Ok(payment)
    }

    async fn get_member_earnings(&self, member_id: &str) -> PoolResult<MemberEarnings> {
        self.member_earnings
            .get(member_id)
            .map(|e| e.clone())
            .ok_or_else(|| {
                // Create a new earnings record if doesn't exist
                let earnings = MemberEarnings::new(member_id.to_string(), self.config.currency);
                self.member_earnings
                    .insert(member_id.to_string(), earnings.clone());
                earnings
            })
            .or_else(|e| Ok(e))
    }

    async fn record_member_payout(&self, member_id: &str, amount: f64) -> PoolResult<()> {
        self.ensure_member_earnings(member_id);

        if let Some(mut earnings) = self.member_earnings.get_mut(member_id) {
            if earnings.pending_payout < amount {
                return Err(PoolError::validation(format!(
                    "Insufficient pending payout: {} < {}",
                    earnings.pending_payout, amount
                )));
            }
            earnings.record_payout(amount);
            Ok(())
        } else {
            Err(PoolError::not_found(format!(
                "Member earnings not found: {}",
                member_id
            )))
        }
    }

    fn get_rate_per_1k(&self, model: &str) -> f64 {
        self.config
            .model_rates
            .get(model)
            .copied()
            .unwrap_or(self.config.base_rate_per_1k)
    }

    fn get_tier_multiplier(&self, tier: SlaTier) -> f64 {
        self.config
            .tier_multipliers
            .get(&tier)
            .copied()
            .unwrap_or(1.0)
    }

    fn get_pool_fee_percent(&self) -> f64 {
        self.config.pool_fee_percent
    }

    fn get_currency(&self) -> Currency {
        self.config.currency
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_billing_service() -> InMemoryBillingService {
        InMemoryBillingService::new(BillingConfig::default())
    }

    #[tokio::test]
    async fn test_create_ticket() {
        let service = create_billing_service();

        let tokens = TokenUsage::new(1000, 500);
        let ticket = service
            .create_ticket(
                "job-1",
                "client-1",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();

        assert_eq!(ticket.job_id, "job-1");
        assert_eq!(ticket.client_id, "client-1");
        assert_eq!(ticket.member_id, "member-1");
        assert_eq!(ticket.tokens_used.total_tokens, 1500);
        assert_eq!(ticket.status, TicketStatus::Pending);
        assert!(ticket.cost.final_amount > 0.0);
    }

    #[tokio::test]
    async fn test_get_client_tickets() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(100, 50);

        // Create multiple tickets
        service
            .create_ticket(
                "job-1",
                "client-1",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();
        service
            .create_ticket(
                "job-2",
                "client-1",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();
        service
            .create_ticket(
                "job-3",
                "client-2",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();

        let client1_tickets = service.get_client_tickets("client-1").await.unwrap();
        let client2_tickets = service.get_client_tickets("client-2").await.unwrap();

        assert_eq!(client1_tickets.len(), 2);
        assert_eq!(client2_tickets.len(), 1);
    }

    #[tokio::test]
    async fn test_create_invoice() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(10000, 5000); // Large enough for min invoice

        // Create tickets
        for i in 0..5 {
            service
                .create_ticket(
                    &format!("job-{}", i),
                    "client-1",
                    "member-1",
                    "llama-7b",
                    tokens,
                    SlaTier::Basic,
                )
                .await
                .unwrap();
        }

        let invoice = service.create_invoice("client-1").await.unwrap();

        assert_eq!(invoice.client_id, "client-1");
        assert_eq!(invoice.tickets.len(), 5);
        assert_eq!(invoice.status, InvoiceStatus::Pending);
        assert!(invoice.total_amount > 0.0);

        // Check tickets are now invoiced
        let pending = service.get_pending_tickets("client-1").await.unwrap();
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_payment_flow() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(10000, 5000);

        // Create tickets and invoice
        for i in 0..3 {
            service
                .create_ticket(
                    &format!("job-{}", i),
                    "client-1",
                    "member-1",
                    "llama-7b",
                    tokens,
                    SlaTier::Basic,
                )
                .await
                .unwrap();
        }

        let invoice = service.create_invoice("client-1").await.unwrap();

        // Record payment
        let payment = service
            .record_payment(
                &invoice.invoice_id,
                invoice.total_amount,
                PaymentMethod::Lightning,
                Some("hash123".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(payment.status, PaymentStatus::Pending);

        // Confirm payment
        let confirmed = service
            .confirm_payment(&payment.payment_id, Some("preimage123".to_string()))
            .await
            .unwrap();

        assert_eq!(confirmed.status, PaymentStatus::Confirmed);
        assert!(confirmed.confirmed_at.is_some());

        // Check invoice is paid
        let invoice = service.get_invoice(&invoice.invoice_id).await.unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Paid);
    }

    #[tokio::test]
    async fn test_member_earnings() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(1000, 500);

        // Create ticket (updates member earnings)
        service
            .create_ticket(
                "job-1",
                "client-1",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();

        let earnings = service.get_member_earnings("member-1").await.unwrap();

        assert_eq!(earnings.jobs_completed, 1);
        assert_eq!(earnings.tokens_processed, 1500);
        assert!(earnings.total_earned > 0.0);
        assert!(earnings.pending_payout > 0.0);
    }

    #[tokio::test]
    async fn test_member_payout() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(10000, 5000);

        // Create multiple tickets
        for i in 0..5 {
            service
                .create_ticket(
                    &format!("job-{}", i),
                    "client-1",
                    "member-1",
                    "llama-7b",
                    tokens,
                    SlaTier::Basic,
                )
                .await
                .unwrap();
        }

        let earnings = service.get_member_earnings("member-1").await.unwrap();
        let payout_amount = earnings.pending_payout / 2.0;

        service
            .record_member_payout("member-1", payout_amount)
            .await
            .unwrap();

        let updated = service.get_member_earnings("member-1").await.unwrap();
        assert!((updated.pending_payout - payout_amount).abs() < 0.01);
        assert_eq!(updated.last_payout_amount, payout_amount);
    }

    #[tokio::test]
    async fn test_tier_multiplier_affects_cost() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(1000, 0);

        let basic_ticket = service
            .create_ticket(
                "job-1",
                "client-1",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();

        let enterprise_ticket = service
            .create_ticket(
                "job-2",
                "client-2",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Enterprise,
            )
            .await
            .unwrap();

        // Enterprise should pay less (0.8 multiplier vs 1.0)
        assert!(enterprise_ticket.cost.final_amount < basic_ticket.cost.final_amount);
    }

    #[tokio::test]
    async fn test_invoice_requires_minimum_amount() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(1, 0); // Very small

        service
            .create_ticket(
                "job-1",
                "client-1",
                "member-1",
                "llama-7b",
                tokens,
                SlaTier::Basic,
            )
            .await
            .unwrap();

        let result = service.create_invoice("client-1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cannot_invoice_twice() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(10000, 5000);

        for i in 0..3 {
            service
                .create_ticket(
                    &format!("job-{}", i),
                    "client-1",
                    "member-1",
                    "llama-7b",
                    tokens,
                    SlaTier::Basic,
                )
                .await
                .unwrap();
        }

        // First invoice succeeds
        service.create_invoice("client-1").await.unwrap();

        // Second invoice fails (no pending tickets)
        let result = service.create_invoice("client-1").await;
        assert!(result.is_err());
    }
}






