//! Creation types for the import workflow (inputs to `*Repository::insert*`).
//! No `id` or timestamps (assigned by the DB / Clock in infra).

use chrono::NaiveDate;

use ffm_domain::ids::{BatchId, Ff3AccountId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, HeaderSignature, MovementStatus};

#[derive(Debug, Clone)]
pub struct NewBatch {
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<Ff3AccountId>,
    pub status: BatchStatus,
}

#[derive(Debug, Clone)]
pub struct NewMovement {
    pub batch_id: BatchId,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub status: MovementStatus,
}

#[derive(Debug, Clone)]
pub struct NewCsvMapping {
    pub header_signature: HeaderSignature,
    pub mapping_json: String,
}
