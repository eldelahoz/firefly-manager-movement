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
    pub synced_at: NaiveDateTime,
}
