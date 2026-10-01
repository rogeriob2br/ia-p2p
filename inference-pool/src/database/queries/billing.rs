//! Billing queries.

use chrono::{DateTime, Utc};
use sqlx::{Any, Row};

use crate::billing::{
    BillingTicket, Cost, Currency, Invoice, InvoiceStatus, TicketStatus, TokenUsage, UsageSummary,
};
use crate::error::{PoolError, PoolResult};
use crate::models::SlaTier;

/// Queries for billing
pub struct BillingQueries;

impl BillingQueries {
    // ========================================================================
    // Billing Tickets
    // ========================================================================

    /// Insert a billing ticket
    pub async fn insert_ticket(pool: &sqlx::Pool<Any>, ticket: &BillingTicket) -> PoolResult<()> {
        let cost_json = serde_json::to_string(&ticket.cost)
            .map_err(|e| PoolError::internal(format!("Failed to serialize cost: {}", e)))?;

        let tokens_json = serde_json::to_string(&ticket.tokens_used)
            .map_err(|e| PoolError::internal(format!("Failed to serialize tokens: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO billing_tickets (
                ticket_id, client_id, member_id, job_id, timestamp, model,
                tokens_input, tokens_output,
                client_cost_amount, client_cost_currency,
                member_payment_amount, member_payment_currency,
                pool_fee_amount, pool_fee_currency,
                payment_status, channel_id, payment_hash, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&ticket.ticket_id)
        .bind(&ticket.client_id)
        .bind(&ticket.member_id)
        .bind(&ticket.job_id)
        .bind(ticket.created_at.to_rfc3339())
        .bind(&ticket.model)
        .bind(ticket.tokens_used.prompt_tokens as i64)
        .bind(ticket.tokens_used.completion_tokens as i64)
        .bind(ticket.cost.final_amount)
        .bind(currency_to_str(&ticket.cost.currency))
        .bind(ticket.cost.member_amount)
        .bind(currency_to_str(&ticket.cost.currency))
        .bind(ticket.cost.pool_fee_amount)
        .bind(currency_to_str(&ticket.cost.currency))
        .bind(ticket_status_to_str(&ticket.status))
        .bind::<Option<&str>>(None) // channel_id
        .bind::<Option<&str>>(None) // payment_hash
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to insert ticket: {}", e)))?;

        Ok(())
    }

    /// Get a ticket by ID
    pub async fn get_ticket(pool: &sqlx::Pool<Any>, ticket_id: &str) -> PoolResult<Option<BillingTicket>> {
        let row = sqlx::query("SELECT * FROM billing_tickets WHERE ticket_id = ?")
            .bind(ticket_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get ticket: {}", e)))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_ticket(&row)?)),
            None => Ok(None),
        }
    }

    /// Get tickets by client
    pub async fn get_tickets_by_client(
        pool: &sqlx::Pool<Any>,
        client_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> PoolResult<Vec<BillingTicket>> {
        let rows = sqlx::query(
            "SELECT * FROM billing_tickets WHERE client_id = ? AND timestamp >= ? AND timestamp <= ? ORDER BY timestamp"
        )
        .bind(client_id)
        .bind(from.to_rfc3339())
        .bind(to.to_rfc3339())
        .fetch_all(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to get tickets: {}", e)))?;

        let tickets: Result<Vec<_>, _> = rows.iter().map(Self::row_to_ticket).collect();
        tickets
    }

    /// Get pending tickets by client
    pub async fn get_pending_tickets_by_client(
        pool: &sqlx::Pool<Any>,
        client_id: &str,
    ) -> PoolResult<Vec<BillingTicket>> {
        let rows = sqlx::query(
            "SELECT * FROM billing_tickets WHERE client_id = ? AND payment_status = 'pending' ORDER BY timestamp"
        )
        .bind(client_id)
        .fetch_all(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to get pending tickets: {}", e)))?;

        let tickets: Result<Vec<_>, _> = rows.iter().map(Self::row_to_ticket).collect();
        tickets
    }

    /// Update ticket status
    pub async fn update_ticket_status(
        pool: &sqlx::Pool<Any>,
        ticket_id: &str,
        status: &TicketStatus,
    ) -> PoolResult<()> {
        sqlx::query("UPDATE billing_tickets SET payment_status = ? WHERE ticket_id = ?")
            .bind(ticket_status_to_str(status))
            .bind(ticket_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update ticket status: {}", e)))?;

        Ok(())
    }

    fn row_to_ticket(row: &sqlx::any::AnyRow) -> PoolResult<BillingTicket> {
        let timestamp_str: String = row.try_get("timestamp").unwrap_or_default();
        let created_at = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let prompt_tokens = row.try_get::<i64, _>("tokens_input").unwrap_or(0) as u64;
        let completion_tokens = row.try_get::<i64, _>("tokens_output").unwrap_or(0) as u64;
        let tokens_used = TokenUsage::new(prompt_tokens, completion_tokens);

        let final_amount: f64 = row.try_get("client_cost_amount").unwrap_or(0.0);
        let member_amount: f64 = row.try_get("member_payment_amount").unwrap_or(0.0);
        let pool_fee_amount: f64 = row.try_get("pool_fee_amount").unwrap_or(0.0);
        let currency = str_to_currency(&row.try_get::<String, _>("client_cost_currency").unwrap_or_default());

        let cost = Cost {
            base_amount: final_amount,
            tier_multiplier: 1.0,
            final_amount,
            currency,
            pool_fee_percent: if final_amount > 0.0 { pool_fee_amount / final_amount * 100.0 } else { 10.0 },
            pool_fee_amount,
            member_amount,
        };

        let status = str_to_ticket_status(&row.try_get::<String, _>("payment_status").unwrap_or_default());

        Ok(BillingTicket {
            ticket_id: row.try_get("ticket_id").unwrap_or_default(),
            job_id: row.try_get("job_id").unwrap_or_default(),
            client_id: row.try_get("client_id").unwrap_or_default(),
            member_id: row.try_get("member_id").unwrap_or_default(),
            model: row.try_get("model").unwrap_or_default(),
            tokens_used,
            cost,
            sla_tier: SlaTier::Free, // Not stored in DB, default
            status,
            created_at,
            finalized_at: None,
            invoice_id: None,
        })
    }

    // ========================================================================
    // Invoices
    // ========================================================================

    /// Insert an invoice
    pub async fn insert_invoice(pool: &sqlx::Pool<Any>, invoice: &Invoice) -> PoolResult<()> {
        let tickets_json = serde_json::to_string(&invoice.tickets)
            .map_err(|e| PoolError::internal(format!("Failed to serialize tickets: {}", e)))?;

        let usage_json = serde_json::to_string(&invoice.usage_summary)
            .map_err(|e| PoolError::internal(format!("Failed to serialize usage: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO invoices (
                invoice_id, client_id, period_start, period_end, ticket_ids,
                subtotal_amount, subtotal_currency,
                tax_amount, tax_currency,
                total_amount, total_currency,
                status, due_date, paid_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&invoice.invoice_id)
        .bind(&invoice.client_id)
        .bind(invoice.created_at.to_rfc3339()) // period_start
        .bind(invoice.due_at.to_rfc3339())     // period_end
        .bind(&tickets_json)
        .bind(invoice.total_amount)
        .bind(currency_to_str(&invoice.currency))
        .bind::<Option<f64>>(None)  // tax_amount
        .bind::<Option<&str>>(None) // tax_currency
        .bind(invoice.total_amount)
        .bind(currency_to_str(&invoice.currency))
        .bind(invoice_status_to_str(&invoice.status))
        .bind(invoice.due_at.to_rfc3339())
        .bind(invoice.paid_at.map(|dt| dt.to_rfc3339()))
        .bind(invoice.created_at.to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| PoolError::internal(format!("Failed to insert invoice: {}", e)))?;

        Ok(())
    }

    /// Get an invoice by ID
    pub async fn get_invoice(pool: &sqlx::Pool<Any>, invoice_id: &str) -> PoolResult<Option<Invoice>> {
        let row = sqlx::query("SELECT * FROM invoices WHERE invoice_id = ?")
            .bind(invoice_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get invoice: {}", e)))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_invoice(&row)?)),
            None => Ok(None),
        }
    }

    /// Get invoices by client
    pub async fn get_invoices_by_client(pool: &sqlx::Pool<Any>, client_id: &str) -> PoolResult<Vec<Invoice>> {
        let rows = sqlx::query("SELECT * FROM invoices WHERE client_id = ? ORDER BY created_at DESC")
            .bind(client_id)
            .fetch_all(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to get invoices: {}", e)))?;

        let invoices: Result<Vec<_>, _> = rows.iter().map(Self::row_to_invoice).collect();
        invoices
    }

    /// Update invoice status
    pub async fn update_invoice_status(
        pool: &sqlx::Pool<Any>,
        invoice_id: &str,
        status: &InvoiceStatus,
        paid_at: Option<DateTime<Utc>>,
    ) -> PoolResult<()> {
        sqlx::query("UPDATE invoices SET status = ?, paid_at = ? WHERE invoice_id = ?")
            .bind(invoice_status_to_str(status))
            .bind(paid_at.map(|dt| dt.to_rfc3339()))
            .bind(invoice_id)
            .execute(pool)
            .await
            .map_err(|e| PoolError::internal(format!("Failed to update invoice status: {}", e)))?;

        Ok(())
    }

    fn row_to_invoice(row: &sqlx::any::AnyRow) -> PoolResult<Invoice> {
        let due_date_str: String = row.try_get("due_date").unwrap_or_default();
        let created_at_str: String = row.try_get("created_at").unwrap_or_default();
        let paid_at_str: Option<String> = row.try_get("paid_at").ok();
        let tickets_json: String = row.try_get("ticket_ids").unwrap_or_default();

        let due_at = DateTime::parse_from_rfc3339(&due_date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let paid_at = paid_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        let tickets: Vec<String> = serde_json::from_str(&tickets_json).unwrap_or_default();
        let currency = str_to_currency(&row.try_get::<String, _>("total_currency").unwrap_or_default());

        Ok(Invoice {
            invoice_id: row.try_get("invoice_id").unwrap_or_default(),
            client_id: row.try_get("client_id").unwrap_or_default(),
            tickets,
            total_amount: row.try_get("total_amount").unwrap_or(0.0),
            currency,
            status: str_to_invoice_status(&row.try_get::<String, _>("status").unwrap_or_default()),
            created_at,
            due_at,
            paid_at,
            payment_reference: None,
            usage_summary: UsageSummary::default(),
        })
    }
}

// Helper functions
fn currency_to_str(c: &Currency) -> &'static str {
    match c {
        Currency::Usd => "USD",
        Currency::Eur => "EUR",
        Currency::Satoshis => "SATS",
    }
}

fn str_to_currency(s: &str) -> Currency {
    match s.to_uppercase().as_str() {
        "USD" => Currency::Usd,
        "EUR" => Currency::Eur,
        "SATS" | "BTC" => Currency::Satoshis,
        _ => Currency::Satoshis,
    }
}

fn ticket_status_to_str(s: &TicketStatus) -> &'static str {
    match s {
        TicketStatus::Pending => "pending",
        TicketStatus::Invoiced => "invoiced",
        TicketStatus::Paid => "paid",
        TicketStatus::Failed => "failed",
        TicketStatus::Cancelled => "cancelled",
    }
}

fn str_to_ticket_status(s: &str) -> TicketStatus {
    match s {
        "pending" => TicketStatus::Pending,
        "invoiced" => TicketStatus::Invoiced,
        "paid" => TicketStatus::Paid,
        "failed" => TicketStatus::Failed,
        "cancelled" => TicketStatus::Cancelled,
        _ => TicketStatus::Pending,
    }
}

fn invoice_status_to_str(s: &InvoiceStatus) -> &'static str {
    match s {
        InvoiceStatus::Draft => "draft",
        InvoiceStatus::Pending => "pending",
        InvoiceStatus::Paid => "paid",
        InvoiceStatus::Overdue => "overdue",
        InvoiceStatus::Cancelled => "cancelled",
    }
}

fn str_to_invoice_status(s: &str) -> InvoiceStatus {
    match s {
        "draft" => InvoiceStatus::Draft,
        "pending" => InvoiceStatus::Pending,
        "paid" => InvoiceStatus::Paid,
        "overdue" => InvoiceStatus::Overdue,
        "cancelled" => InvoiceStatus::Cancelled,
        _ => InvoiceStatus::Pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{DatabaseConfig, DatabasePool, MigrationManager};

    async fn setup_db() -> sqlx::Pool<Any> {
        let config = DatabaseConfig::sqlite_memory();
        let pool = DatabasePool::new(config).await.unwrap();
        let manager = MigrationManager::new(pool);
        manager.migrate_up().await.unwrap();
        manager.pool.pool().clone()
    }

    fn create_test_ticket() -> BillingTicket {
        BillingTicket {
            ticket_id: "ticket-1".to_string(),
            job_id: "job-1".to_string(),
            client_id: "client-1".to_string(),
            member_id: "member-1".to_string(),
            model: "gpt-4".to_string(),
            tokens_used: TokenUsage::new(100, 50),
            cost: Cost::calculate(&TokenUsage::new(100, 50), 10.0, 1.0, 10.0, Currency::Satoshis),
            sla_tier: SlaTier::Free,
            status: TicketStatus::Pending,
            created_at: Utc::now(),
            finalized_at: None,
            invoice_id: None,
        }
    }

    #[tokio::test]
    async fn test_insert_and_get_ticket() {
        let pool = setup_db().await;
        let ticket = create_test_ticket();

        BillingQueries::insert_ticket(&pool, &ticket).await.unwrap();

        let loaded = BillingQueries::get_ticket(&pool, "ticket-1").await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.client_id, "client-1");
        assert_eq!(loaded.tokens_used.prompt_tokens, 100);
    }

    #[tokio::test]
    async fn test_update_ticket_status() {
        let pool = setup_db().await;
        BillingQueries::insert_ticket(&pool, &create_test_ticket()).await.unwrap();

        BillingQueries::update_ticket_status(&pool, "ticket-1", &TicketStatus::Paid)
            .await
            .unwrap();

        let loaded = BillingQueries::get_ticket(&pool, "ticket-1").await.unwrap().unwrap();
        assert_eq!(loaded.status, TicketStatus::Paid);
    }
}
