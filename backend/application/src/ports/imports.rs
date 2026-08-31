use std::collections::HashMap;

use async_trait::async_trait;

use ffm_domain::entities::{Batch, CsvMapping, Movement};
use ffm_domain::ids::{BatchId, CsvMappingId, Ff3AccountId, MovementId};
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};

use crate::dtos::{NewBatch, NewCsvMapping, NewMovement};
use crate::error::ApplicationError;

#[async_trait]
pub trait BatchRepository: Send + Sync {
    async fn insert(&self, batch: NewBatch) -> Result<BatchId, ApplicationError>;
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError>;
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError>;
    async fn update_status(&self, id: BatchId, status: BatchStatus)
        -> Result<(), ApplicationError>;
    async fn update_account(
        &self,
        id: BatchId,
        account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError>;
    /// Batches cuyo filename coincide (MySQL collation es case-insensitive).
    /// Default: sin duplicados (fakes que no lo implementan).
    async fn find_by_filename(&self, _filename: &str) -> Result<Vec<Batch>, ApplicationError> {
        Ok(Vec::new())
    }
    /// Elimina un batch por id. Default: no soportado (fakes que no lo implementan).
    async fn delete(&self, _id: BatchId) -> Result<(), ApplicationError> {
        Err(ApplicationError::InvalidState(
            "delete not supported by this repository".into(),
        ))
    }
}

#[async_trait]
pub trait MovementRepository: Send + Sync {
    async fn insert_batch(&self, movements: Vec<NewMovement>) -> Result<(), ApplicationError>;
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError>;
    async fn count_by_batch(&self) -> Result<HashMap<BatchId, i64>, ApplicationError> {
        Ok(HashMap::new())
    }
    async fn update_status(
        &self,
        id: MovementId,
        status: MovementStatus,
    ) -> Result<(), ApplicationError>;
    /// Elimina todos los movimientos de un batch. Default: no soportado.
    async fn delete_by_batch(&self, _batch_id: BatchId) -> Result<(), ApplicationError> {
        Err(ApplicationError::InvalidState(
            "delete_by_batch not supported by this repository".into(),
        ))
    }
}

#[async_trait]
pub trait CsvMappingRepository: Send + Sync {
    async fn by_signature(
        &self,
        signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError>;
    async fn upsert(&self, mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError>;
}

/// File decoder (encoding + delimiter) - infra detail.
#[async_trait]
pub trait CsvDecoder: Send + Sync {
    async fn decode(&self, bytes: Vec<u8>) -> Result<String, ApplicationError>;
}
