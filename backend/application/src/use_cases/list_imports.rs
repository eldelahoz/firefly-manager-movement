//! ListImports use case (RF-09).

use crate::error::ApplicationError;
use crate::ports::BatchRepository;

use ffm_domain::entities::Batch;

pub struct ListImports<R> {
    batches: R,
}

impl<R> ListImports<R> {
    pub fn new(batches: R) -> Self {
        Self { batches }
    }
}

impl<R: BatchRepository> ListImports<R> {
    pub async fn execute(&self) -> Result<Vec<Batch>, ApplicationError> {
        self.batches.list().await
    }
}
