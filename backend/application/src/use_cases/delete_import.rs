//! DeleteImport use case.
//! Elimina un batch en estado `parsed` junto con sus movimientos.
//! Solo `parsed`: los batches reconciled/applied arrastran reconciliaciones,
//! matches y (en applied) transacciones vivas en Firefly III que no deben
//! borrarse en silencio.

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
