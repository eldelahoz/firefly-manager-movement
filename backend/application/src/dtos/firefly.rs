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
    pub iban: Option<String>,
    pub account_number: Option<String>,
    /// `account_role` of asset accounts (`defaultAsset`, `ccAsset`, …).
    pub role: Option<String>,
    /// `loan`, `debt` or `mortgage` for liabilities.
    pub liability_type: Option<String>,
}

/// Firefly III transaction as returned by the API (no local ID).
#[derive(Debug, Clone)]
pub struct Ff3TransactionDto {
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    /// In-window splits on the queried account when the group has 2 or
    /// more; empty otherwise. `amount_cents` is their total.
    pub splits: Vec<Ff3SplitDto>,
    /// Account on the opposite side of the queried account (RF-14 v0.2):
    /// its id (RF-08 v0.11), name and Firefly III type (`Expense account`, `Loan`, …).
    pub counterparty_ff3_id: Option<i64>,
    pub counterparty_name: Option<String>,
    pub counterparty_type: Option<String>,
}

/// One journal of a split transaction group, signed for the queried account.
#[derive(Debug, Clone)]
pub struct Ff3SplitDto {
    pub journal_id: i64,
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
    pub counterparty_name: Option<String>,
    pub counterparty_type: Option<String>,
}

/// A transaction just created in Firefly III: its id and both account
/// sides as Firefly III stored them (names of accounts it created included).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreatedFf3Transaction {
    pub ff3_id: i64,
    pub source_id: Option<i64>,
    pub destination_id: Option<i64>,
    pub source_name: Option<String>,
    pub source_type: Option<String>,
    pub destination_name: Option<String>,
    pub destination_type: Option<String>,
}

impl From<i64> for CreatedFf3Transaction {
    fn from(ff3_id: i64) -> Self {
        Self {
            ff3_id,
            ..Default::default()
        }
    }
}

/// Data to create a transaction in Firefly III (RF-08/RF-11).
#[derive(Debug, Clone)]
pub struct NewFf3TransactionCreation {
    pub transaction_type: String,
    pub amount: AmountCents,
    pub date: NaiveDate,
    pub description: String,
    pub source_ff3_id: Option<i64>,
    pub destination_ff3_id: Option<i64>,
    /// Account Firefly III creates by name (`withdrawal`/`deposit` only).
    pub source_name: Option<String>,
    pub destination_name: Option<String>,
    pub category: Option<String>,
    pub budget: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    /// Stable id stored on the split so a retry can find it (NFR-05).
    pub external_id: Option<String>,
}

/// A split on one account found by its `external_id` (NFR-05 recovery).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalIdHit {
    pub external_id: String,
    pub ff3_id: i64,
    /// The side opposite to the queried account.
    pub counterparty_name: Option<String>,
    pub counterparty_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewFf3Account {
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub iban: Option<String>,
    pub account_number: Option<String>,
}
