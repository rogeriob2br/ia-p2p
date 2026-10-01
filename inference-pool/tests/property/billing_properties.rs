//! Property tests for Billing System.
//!
//! Tests billing invariants and business rules.

use proptest::prelude::*;

use inference_pool::billing::{
    BillingConfig, BillingService, Cost, Currency, InMemoryBillingService, InvoiceStatus,
    PaymentMethod, PaymentStatus, TicketStatus, TokenUsage,
};
use inference_pool::models::SlaTier;

// ============================================================================
// Billing Service Configuration
// ============================================================================

fn create_billing_service() -> InMemoryBillingService {
    InMemoryBillingService::new(BillingConfig::default())
}

// ============================================================================
// Property: Cost Calculation Invariants
// ============================================================================

proptest! {
    /// Cost is always proportional to tokens used
    #[test]
    fn cost_proportional_to_tokens(
        prompt_tokens in 1u64..100_000,
        completion_tokens in 1u64..100_000,
    ) {
        let tokens1 = TokenUsage::new(prompt_tokens, completion_tokens);
        let tokens2 = TokenUsage::new(prompt_tokens * 2, completion_tokens * 2);

        let cost1 = Cost::calculate(&tokens1, 10.0, 1.0, 10.0, Currency::Satoshis);
        let cost2 = Cost::calculate(&tokens2, 10.0, 1.0, 10.0, Currency::Satoshis);

        // Double tokens should mean double cost
        prop_assert!((cost2.final_amount - cost1.final_amount * 2.0).abs() < 0.01);
    }

    /// Pool fee + member amount = total amount
    #[test]
    fn fee_plus_member_equals_total(
        tokens in 100u64..100_000,
        pool_fee in 0.0f64..50.0,
    ) {
        let usage = TokenUsage::new(tokens, tokens / 2);
        let cost = Cost::calculate(&usage, 10.0, 1.0, pool_fee, Currency::Satoshis);

        let sum = cost.pool_fee_amount + cost.member_amount;
        prop_assert!((sum - cost.final_amount).abs() < 0.001);
    }

    /// Tier multiplier correctly scales final amount
    #[test]
    fn tier_multiplier_scales_amount(
        tokens in 1000u64..10_000,
        multiplier in 0.5f64..2.0,
    ) {
        let usage = TokenUsage::new(tokens, 0);
        let cost = Cost::calculate(&usage, 10.0, multiplier, 10.0, Currency::Satoshis);

        let expected_final = cost.base_amount * multiplier;
        prop_assert!((cost.final_amount - expected_final).abs() < 0.001);
    }
}

// ============================================================================
// Property: Ticket Creation
// ============================================================================

proptest! {
    /// Every ticket gets a unique ID
    #[test]
    fn tickets_have_unique_ids(
        job_count in 2usize..20,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(1000, 500);

            let mut ticket_ids = Vec::new();
            for i in 0..job_count {
                let ticket = service
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
                ticket_ids.push(ticket.ticket_id);
            }

            // All IDs should be unique
            let unique_count = ticket_ids.iter().collect::<std::collections::HashSet<_>>().len();
            prop_assert_eq!(unique_count, job_count);

            Ok(())
        })?;
    }

    /// Ticket cost is never negative
    #[test]
    fn ticket_cost_never_negative(
        prompt in 0u64..100_000,
        completion in 0u64..100_000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(prompt, completion);

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

            prop_assert!(ticket.cost.final_amount >= 0.0);
            prop_assert!(ticket.cost.pool_fee_amount >= 0.0);
            prop_assert!(ticket.cost.member_amount >= 0.0);

            Ok(())
        })?;
    }
}

// ============================================================================
// Property: Member Earnings
// ============================================================================

proptest! {
    /// Member earnings increase with each job
    #[test]
    fn member_earnings_monotonic(
        job_count in 1usize..10,
        tokens_per_job in 1000u64..10_000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(tokens_per_job, tokens_per_job / 2);

            let mut last_total = 0.0;
            for i in 0..job_count {
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

                let earnings = service.get_member_earnings("member-1").await.unwrap();
                prop_assert!(earnings.total_earned >= last_total);
                last_total = earnings.total_earned;
            }

            Ok(())
        })?;
    }

    /// Payout reduces pending amount correctly
    #[test]
    fn payout_reduces_pending(
        job_count in 5usize..10,
        payout_percent in 10u32..90,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(10000, 5000);

            // Create jobs
            for i in 0..job_count {
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

            let before = service.get_member_earnings("member-1").await.unwrap();
            let payout_amount = before.pending_payout * (payout_percent as f64 / 100.0);

            service.record_member_payout("member-1", payout_amount).await.unwrap();

            let after = service.get_member_earnings("member-1").await.unwrap();

            // Pending should decrease by payout amount
            let diff = before.pending_payout - after.pending_payout;
            prop_assert!((diff - payout_amount).abs() < 0.01);

            // Total earned shouldn't change
            prop_assert!((after.total_earned - before.total_earned).abs() < 0.01);

            Ok(())
        })?;
    }
}

// ============================================================================
// Property: Invoice Creation
// ============================================================================

proptest! {
    /// Invoice total equals sum of ticket costs
    #[test]
    fn invoice_total_equals_sum(
        job_count in 3usize..10,
        tokens_per_job in 5000u64..20_000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(tokens_per_job, tokens_per_job / 2);

            let mut expected_total = 0.0;
            for i in 0..job_count {
                let ticket = service
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
                expected_total += ticket.cost.final_amount;
            }

            let invoice = service.create_invoice("client-1").await.unwrap();

            prop_assert!((invoice.total_amount - expected_total).abs() < 0.01);
            prop_assert_eq!(invoice.tickets.len(), job_count);

            Ok(())
        })?;
    }

    /// Cannot create invoice from empty pending tickets
    #[test]
    fn cannot_invoice_empty_pending(_x in 0..1) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();

            let result = service.create_invoice("client-1").await;
            prop_assert!(result.is_err());

            Ok(())
        })?;
    }
}

// ============================================================================
// Property: Payment Flow
// ============================================================================

proptest! {
    /// Payment confirmation marks invoice as paid
    #[test]
    fn payment_confirms_invoice(
        job_count in 3usize..8,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(10000, 5000);

            for i in 0..job_count {
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
            prop_assert_eq!(invoice.status, InvoiceStatus::Pending);

            let payment = service
                .record_payment(
                    &invoice.invoice_id,
                    invoice.total_amount,
                    PaymentMethod::Lightning,
                    Some("hash123".to_string()),
                )
                .await
                .unwrap();

            prop_assert_eq!(payment.status, PaymentStatus::Pending);

            let confirmed = service
                .confirm_payment(&payment.payment_id, Some("preimage123".to_string()))
                .await
                .unwrap();

            prop_assert_eq!(confirmed.status, PaymentStatus::Confirmed);

            let updated_invoice = service.get_invoice(&invoice.invoice_id).await.unwrap();
            prop_assert_eq!(updated_invoice.status, InvoiceStatus::Paid);
            prop_assert!(updated_invoice.paid_at.is_some());

            Ok(())
        })?;
    }

    /// Tickets are marked paid after invoice payment
    #[test]
    fn tickets_paid_after_invoice_payment(
        job_count in 2usize..5,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let tokens = TokenUsage::new(10000, 5000);

            let mut ticket_ids = Vec::new();
            for i in 0..job_count {
                let ticket = service
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
                ticket_ids.push(ticket.ticket_id);
            }

            let invoice = service.create_invoice("client-1").await.unwrap();

            // Check tickets are invoiced
            for ticket_id in &ticket_ids {
                let ticket = service.get_ticket(ticket_id).await.unwrap();
                prop_assert_eq!(ticket.status, TicketStatus::Invoiced);
            }

            // Complete payment
            let payment = service
                .record_payment(
                    &invoice.invoice_id,
                    invoice.total_amount,
                    PaymentMethod::Lightning,
                    None,
                )
                .await
                .unwrap();

            service.confirm_payment(&payment.payment_id, None).await.unwrap();

            // Check tickets are paid
            for ticket_id in &ticket_ids {
                let ticket = service.get_ticket(ticket_id).await.unwrap();
                prop_assert_eq!(ticket.status, TicketStatus::Paid);
                prop_assert!(ticket.finalized_at.is_some());
            }

            Ok(())
        })?;
    }
}

// ============================================================================
// Property: SLA Tier Affects Pricing
// ============================================================================

proptest! {
    /// Enterprise tier pays less than Basic tier
    #[test]
    fn enterprise_pays_less(
        tokens in 5000u64..50_000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = create_billing_service();
            let usage = TokenUsage::new(tokens, tokens / 2);

            let basic = service
                .create_ticket("job-1", "client-1", "member-1", "llama-7b", usage, SlaTier::Basic)
                .await
                .unwrap();

            let enterprise = service
                .create_ticket("job-2", "client-2", "member-1", "llama-7b", usage, SlaTier::Enterprise)
                .await
                .unwrap();

            // Enterprise should be cheaper (0.8 multiplier vs 1.0)
            prop_assert!(enterprise.cost.final_amount < basic.cost.final_amount);

            // Pro should be between
            let pro = service
                .create_ticket("job-3", "client-3", "member-1", "llama-7b", usage, SlaTier::Pro)
                .await
                .unwrap();

            prop_assert!(pro.cost.final_amount < basic.cost.final_amount);
            prop_assert!(pro.cost.final_amount > enterprise.cost.final_amount);

            Ok(())
        })?;
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

pub mod unit_tests {
    use super::*;

    #[test]
    fn test_currency_properties() {
        assert_eq!(Currency::Satoshis.decimals(), 0);
        assert_eq!(Currency::Usd.decimals(), 2);
        assert!(!Currency::Satoshis.symbol().is_empty());
    }

    #[tokio::test]
    async fn test_usage_summary_in_invoice() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(10000, 5000);

        // Create tickets with different models
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

        for i in 0..2 {
            service
                .create_ticket(
                    &format!("job-b-{}", i),
                    "client-1",
                    "member-1",
                    "mistral-7b",
                    tokens,
                    SlaTier::Basic,
                )
                .await
                .unwrap();
        }

        let invoice = service.create_invoice("client-1").await.unwrap();

        assert_eq!(invoice.usage_summary.total_requests, 5);
        assert_eq!(invoice.usage_summary.by_model.len(), 2);
        assert_eq!(invoice.usage_summary.by_model.get("llama-7b").unwrap().requests, 3);
        assert_eq!(invoice.usage_summary.by_model.get("mistral-7b").unwrap().requests, 2);
    }

    #[tokio::test]
    async fn test_cannot_overpayout() {
        let service = create_billing_service();
        let tokens = TokenUsage::new(1000, 500);

        service
            .create_ticket("job-1", "client-1", "member-1", "llama-7b", tokens, SlaTier::Basic)
            .await
            .unwrap();

        let earnings = service.get_member_earnings("member-1").await.unwrap();

        // Try to payout more than pending
        let result = service
            .record_member_payout("member-1", earnings.pending_payout * 2.0)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_payment_amount_must_match() {
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

        let invoice = service.create_invoice("client-1").await.unwrap();

        // Try to pay wrong amount
        let result = service
            .record_payment(
                &invoice.invoice_id,
                invoice.total_amount + 100.0, // Wrong amount
                PaymentMethod::Lightning,
                None,
            )
            .await;

        assert!(result.is_err());
    }
}

