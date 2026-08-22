use chrono::{NaiveDate, NaiveDateTime};

use crate::ids::{BatchId, ReconciliationId};

/// Frozen result of a rapprochement run (audit snapshot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    pub id: ReconciliationId,
    pub batch_id: BatchId,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
    pub ran_at: NaiveDateTime,
}
