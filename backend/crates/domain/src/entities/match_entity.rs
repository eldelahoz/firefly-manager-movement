use chrono::NaiveDateTime;

use crate::ids::{Ff3TransactionId, MatchId, MovementId, ReconciliationId};

/// One-to-one pairing historized per run (the 1-to-1 is guaranteed by the schema).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub id: MatchId,
    pub reconciliation_id: ReconciliationId,
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
    pub matched_at: NaiveDateTime,
}
