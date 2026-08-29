//! Firefly III client DTOs (external API shape, no local IDs).

use chrono::NaiveDate;

use ffm_domain::value_objects::AmountCents;

/// Response of `GET /api/v1/about` (FF3 health).
#[derive(Debug, Clone)]
pub struct About {
    pub version: String,
}

/// Asset account as returned by the Firefly III API (no local ID).
#[derive(Debug, Clone)]
pub struct Ff3AccountDto {
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
}

/// Firefly III transaction as returned by the API (no local ID).
#[derive(Debug, Clone)]
pub struct Ff3TransactionDto {
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
}

/// Reconciliation status of an FF3 transaction, derived from the local
/// run cache (browse widget enrichment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ff3TxStatus {
    Matched,
    Duplicate,
    Extra,
}

impl Ff3TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ff3TxStatus::Matched => "matched",
            Ff3TxStatus::Duplicate => "duplicate",
            Ff3TxStatus::Extra => "extra",
        }
    }
}

/// Live FF3 transaction enriched with its cached reconciliation status:
/// `duplicate` when the latest cache row flags it, else `matched` when it
/// has a match in any run, else `extra` when cached without a match,
/// else `None` (never seen in a reconciliation).
#[derive(Debug, Clone)]
pub struct BrowseTxDto {
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub status: Option<Ff3TxStatus>,
}

/// Data to create a transaction in Firefly III (RF-08/RF-11).
#[derive(Debug, Clone)]
pub struct NewFf3TransactionCreation {
    pub transaction_type: String,
    pub amount: AmountCents,
    pub date: NaiveDate,
    pub description: String,
    pub source_ff3_id: i64,
    pub destination_ff3_id: i64,
    pub category: Option<String>,
    pub budget: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NewFf3Account {
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
}
