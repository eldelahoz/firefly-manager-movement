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
    /// CSV header row, persisted so an unfinished import can be resumed
    /// with its original mapping columns (RF-02, RF-18).
    pub headers: Vec<String>,
    /// Delimiter detected at upload time (single char as string).
    pub detected_delimiter: String,
}
