//! IgnoreMovement use case (RF-10): marks a movement `ignored`.

use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::MovementStatus;

use crate::error::ApplicationError;
use crate::ports::{BatchRepository, MovementRepository};

pub struct IgnoreMovement<M, B> {
    movements: M,
    batches: B,
}

impl<M, B> IgnoreMovement<M, B> {
    pub fn new(movements: M, batches: B) -> Self {
        Self { movements, batches }
    }
}

impl<M, B> IgnoreMovement<M, B>
where
    M: MovementRepository,
    B: BatchRepository,
{
    pub async fn execute(
        &self,
        batch_id: BatchId,
        movement_id: MovementId,
    ) -> Result<(), ApplicationError> {
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

        self.movements
            .update_status(movement_id, MovementStatus::Ignored)
            .await
    }
}
