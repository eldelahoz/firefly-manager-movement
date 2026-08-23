use async_trait::async_trait;
use chrono::NaiveDate;

use ffm_domain::entities::Ff3Account;
use ffm_domain::ids::Ff3AccountId;

use crate::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewFf3Account, NewFf3TransactionCreation,
};
use crate::error::ApplicationError;

/// Port for the Firefly III HTTP client (API v1).
#[async_trait]
pub trait FireflyClient: Send + Sync {
    async fn about(&self) -> Result<About, ApplicationError>;
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError>;
    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError>;
    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError>;
    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait Ff3AccountRepository: Send + Sync {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError>;
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError>;
    async fn by_id(&self, id: Ff3AccountId) -> Result<Option<Ff3Account>, ApplicationError>;
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError>;
}
