use chrono::NaiveDateTime;

use crate::ids::{CreationId, MovementId};

/// Traceability of a transaction we created in Firefly III.
/// `UNIQUE(movement_id)` → idempotency (NFR-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Creation {
    pub id: CreationId,
    pub movement_id: MovementId,
    pub ff3_id: i64,
    pub created_at: NaiveDateTime,
}
