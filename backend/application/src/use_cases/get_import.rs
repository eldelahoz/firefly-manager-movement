//! GetImport use case (RF-09).

use crate::error::ApplicationError;
use crate::ports::{BatchRepository, MovementRepository};

use ffm_domain::entities::{Batch, Movement};
use ffm_domain::ids::BatchId;

pub struct GetImport<B, M> {
    batches: B,
    movements: M,
}

impl<B, M> GetImport<B, M> {
    pub fn new(batches: B, movements: M) -> Self {
        Self { batches, movements }
    }
}

impl<B: BatchRepository, M: MovementRepository> GetImport<B, M> {
    pub async fn execute(
        &self,
        batch_id: BatchId,
    ) -> Result<(Batch, Vec<Movement>), ApplicationError> {
        let batch = self.batches.get(batch_id).await?;
        let movements = self.movements.by_batch(batch_id).await?;
        Ok((batch, movements))
    }
}
