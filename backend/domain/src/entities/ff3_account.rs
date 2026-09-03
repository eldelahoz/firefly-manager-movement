use chrono::NaiveDateTime;

use crate::ids::Ff3AccountId;

/// Local cache of a Firefly III asset account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff3Account {
    pub id: Ff3AccountId,
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    /// IBAN as exported by the bank (origin-account matching, RF-04 v0.7).
    pub iban: Option<String>,
    /// Account number as known by FF3 (origin-account matching, RF-04 v0.7).
    pub account_number: Option<String>,
    pub synced_at: NaiveDateTime,
}
