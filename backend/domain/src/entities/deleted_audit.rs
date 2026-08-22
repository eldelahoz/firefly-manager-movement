use chrono::NaiveDateTime;

use crate::ids::DeletedAuditId;

/// Audit snapshot of a deleted FF3 transaction (point-in-time, denormalized).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedAudit {
    pub id: DeletedAuditId,
    pub ff3_id: i64,
    pub snapshot_json: String,
    pub deleted_at: NaiveDateTime,
}
