use chrono::NaiveDateTime;

use crate::ids::{BatchId, Ff3AccountId};
use crate::value_objects::BatchStatus;

/// Batch of movements imported from a file (immutable, RF-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub id: BatchId,
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<Ff3AccountId>,
    pub status: BatchStatus,
    pub created_at: NaiveDateTime,
}
