//! Creation type for the delete-audit trail.

#[derive(Debug, Clone)]
pub struct NewDeletedAudit {
    pub ff3_id: i64,
    pub snapshot_json: String,
}
