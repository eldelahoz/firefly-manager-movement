//! IgnoreMovement use case (RF-10): sets or clears the `ignored` status.
//!
//! Ignoring and un-ignoring are status changes only — a movement is never
//! removed from the import batch. Un-ignore restores the status implied by
//! the latest run's outcome (`Imported` when no run exists).

use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::MovementStatus;

use crate::error::ApplicationError;
use crate::ports::{
    BatchRepository, MatchRepository, MovementRepository, ReconciliationRepository,
};

pub struct IgnoreMovement<Mov, B, R, Mt> {
    movements: Mov,
    batches: B,
    reconciliations: R,
    matches: Mt,
}

impl<Mov, B, R, Mt> IgnoreMovement<Mov, B, R, Mt> {
    pub fn new(movements: Mov, batches: B, reconciliations: R, matches: Mt) -> Self {
        Self {
            movements,
            batches,
            reconciliations,
            matches,
        }
    }
}

impl<Mov, B, R, Mt> IgnoreMovement<Mov, B, R, Mt>
where
    Mov: MovementRepository,
    B: BatchRepository,
    R: ReconciliationRepository,
    Mt: MatchRepository,
{
    /// `ignored = true` sets `Ignored`; `false` restores the latest-run
    /// status and returns it.
    pub async fn execute(
        &self,
        batch_id: BatchId,
        movement_id: MovementId,
        ignored: bool,
    ) -> Result<MovementStatus, ApplicationError> {
        self.batches.get(batch_id).await?;

        // The movement must belong to the batch (route is batch-scoped).
        self.movements
            .by_batch(batch_id)
            .await?
            .iter()
            .find(|m| m.id == movement_id)
            .ok_or_else(|| {
                ApplicationError::NotFound(format!(
                    "movement {movement_id} not found in batch {batch_id}"
                ))
            })?;

        let target = if ignored {
            MovementStatus::Ignored
        } else {
            self.restore_status(batch_id, movement_id).await?
        };
        self.movements.update_status(movement_id, target).await?;
        Ok(target)
    }

    async fn restore_status(
        &self,
        batch_id: BatchId,
        movement_id: MovementId,
    ) -> Result<MovementStatus, ApplicationError> {
        let Some(run) = self.reconciliations.latest_by_batch(batch_id).await? else {
            return Ok(MovementStatus::Imported);
        };
        let matched = self
            .matches
            .by_reconciliation(run.id)
            .await?
            .iter()
            .any(|m| m.movement_id == movement_id);
        Ok(if matched {
            MovementStatus::Matched
        } else {
            MovementStatus::Missing
        })
    }
}
