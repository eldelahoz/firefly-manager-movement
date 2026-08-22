//! Blanket impls so `Arc<dyn Trait>` can be used in use cases.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;

use ffm_domain::entities::{Batch, CsvMapping, Ff3Account, Movement};
use ffm_domain::ids::{BatchId, CsvMappingId, Ff3AccountId, MovementId};
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};

use crate::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewBatch, NewCsvMapping, NewFf3Account,
    NewFf3TransactionCreation, NewMovement,
};
use crate::error::ApplicationError;

use super::{
    BatchRepository, CsvDecoder, CsvMappingRepository, Ff3AccountRepository, FireflyClient,
    MovementRepository,
};

#[async_trait]
impl FireflyClient for Arc<dyn FireflyClient> {
    async fn about(&self) -> Result<About, ApplicationError> {
        self.as_ref().about().await
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        self.as_ref().asset_accounts().await
    }
    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        self.as_ref()
            .account_transactions(account_ff3_id, start, end)
            .await
    }
    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        self.as_ref().create_transaction(creation).await
    }
    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        self.as_ref().delete_transaction(ff3_id).await
    }
}

#[async_trait]
impl Ff3AccountRepository for Arc<dyn Ff3AccountRepository> {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        self.as_ref().upsert_batch(accounts).await
    }
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        self.as_ref().by_ff3_id(ff3_id).await
    }
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        self.as_ref().all_asset().await
    }
}

#[async_trait]
impl<T: BatchRepository + ?Sized> BatchRepository for Arc<T> {
    async fn insert(&self, batch: NewBatch) -> Result<BatchId, ApplicationError> {
        self.as_ref().insert(batch).await
    }
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError> {
        self.as_ref().get(id).await
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        self.as_ref().list().await
    }
    async fn update_status(
        &self,
        id: BatchId,
        status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        self.as_ref().update_status(id, status).await
    }
    async fn update_account(
        &self,
        id: BatchId,
        account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        self.as_ref().update_account(id, account_id).await
    }
}

#[async_trait]
impl<T: MovementRepository + ?Sized> MovementRepository for Arc<T> {
    async fn insert_batch(&self, movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        self.as_ref().insert_batch(movements).await
    }
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        self.as_ref().by_batch(batch_id).await
    }
    async fn update_status(
        &self,
        id: MovementId,
        status: MovementStatus,
    ) -> Result<(), ApplicationError> {
        self.as_ref().update_status(id, status).await
    }
}

#[async_trait]
impl<T: CsvMappingRepository + ?Sized> CsvMappingRepository for Arc<T> {
    async fn by_signature(
        &self,
        signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError> {
        self.as_ref().by_signature(signature).await
    }
    async fn upsert(&self, mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError> {
        self.as_ref().upsert(mapping).await
    }
}

#[async_trait]
impl<T: CsvDecoder + ?Sized> CsvDecoder for Arc<T> {
    async fn decode(&self, bytes: Vec<u8>) -> Result<String, ApplicationError> {
        self.as_ref().decode(bytes).await
    }
}
