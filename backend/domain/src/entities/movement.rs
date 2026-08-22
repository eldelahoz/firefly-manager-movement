use chrono::{NaiveDate, NaiveDateTime};

use crate::ids::{BatchId, MovementId};
use crate::value_objects::{AmountCents, MovementStatus};

/// Movement imported from a bank statement.
/// `amount_cents` is signed (negative = debit); no `direction` column (TRD D2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    pub id: MovementId,
    pub batch_id: BatchId,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub status: MovementStatus,
    pub created_at: NaiveDateTime,
}
