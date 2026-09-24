//! DeleteImport use case (RF-17): deletes a batch in `parsed` status with
//! its movements. Only `parsed` — `reconciled`/`applied` batches carry
//! reconciliations, matches and (for `applied`) live Firefly III
//! transactions that must not be deleted silently.

use ffm_domain::ids::BatchId;
use ffm_domain::value_objects::BatchStatus;

use crate::error::ApplicationError;
use crate::ports::{BatchRepository, MovementRepository};

pub struct DeleteImport<B, M> {
    batches: B,
    movements: M,
}

impl<B, M> DeleteImport<B, M> {
    pub fn new(batches: B, movements: M) -> Self {
        Self { batches, movements }
    }
}

impl<B, M> DeleteImport<B, M>
where
    B: BatchRepository,
    M: MovementRepository,
{
    pub async fn execute(&self, id: BatchId) -> Result<(), ApplicationError> {
        let batch = self.batches.get(id).await?;
        if batch.status != BatchStatus::Parsed {
            return Err(ApplicationError::InvalidState(
                "solo se pueden eliminar imports en estado 'parsed'".into(),
            ));
        }
        self.movements.delete_by_batch(id).await?;
        self.batches.delete(id).await?;
        Ok(())
    }
}
