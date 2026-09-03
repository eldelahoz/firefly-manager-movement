//! Runtime-swappable FF3 client: settings take precedence over env vars
//! (RF-19); `reconfigure` is called when settings are saved.

use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use chrono::NaiveDate;
use ffm_application::dtos::{About, Ff3AccountDto, Ff3TransactionDto, NewFf3TransactionCreation};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{FireflyClient, FireflyClientFactory, FireflyConnectionManager};

use super::client::ReqwestFireflyClient;

/// A `FireflyClient` that can be reconfigured at runtime.
/// Used when app_settings exist in the database (RF-19).
pub struct DynamicFireflyClient {
    current: RwLock<Arc<ReqwestFireflyClient>>,
}

impl DynamicFireflyClient {
    pub fn new(ff3_url: &str, ff3_token: &str) -> Self {
        Self {
            current: RwLock::new(Arc::new(ReqwestFireflyClient::new(ff3_url, ff3_token))),
        }
    }

    fn current(&self) -> Arc<ReqwestFireflyClient> {
        self.current
            .read()
            .expect("dynamic client lock poisoned")
            .clone()
    }
}

#[async_trait]
impl FireflyConnectionManager for DynamicFireflyClient {
    async fn reconfigure(&self, ff3_url: &str, ff3_token: &str) -> Result<(), ApplicationError> {
        let mut guard = self.current.write().expect("dynamic client lock poisoned");
        *guard = Arc::new(ReqwestFireflyClient::new(ff3_url, ff3_token));
        Ok(())
    }
}

#[async_trait]
impl FireflyClient for DynamicFireflyClient {
    async fn about(&self) -> Result<About, ApplicationError> {
        self.current().about().await
    }

    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        self.current().asset_accounts().await
    }

    async fn accounts_by_type(
        &self,
        account_type: &str,
    ) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        self.current().accounts_by_type(account_type).await
    }

    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        self.current()
            .account_transactions(account_ff3_id, start, end)
            .await
    }

    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        self.current().create_transaction(creation).await
    }

    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        self.current().delete_transaction(ff3_id).await
    }

    async fn tags(&self) -> Result<Vec<String>, ApplicationError> {
        self.current().tags().await
    }

    async fn categories(&self) -> Result<Vec<String>, ApplicationError> {
        self.current().categories().await
    }

    async fn budgets(&self) -> Result<Vec<String>, ApplicationError> {
        self.current().budgets().await
    }

    async fn attach_to_transaction(
        &self,
        journal_ff3_id: i64,
        filename: &str,
        content_base64: &str,
    ) -> Result<(), ApplicationError> {
        self.current()
            .attach_to_transaction(journal_ff3_id, filename, content_base64)
            .await
    }
}

/// Factory that builds `ReqwestFireflyClient` instances.
pub struct ReqwestFireflyClientFactory;

impl FireflyClientFactory for ReqwestFireflyClientFactory {
    fn build(&self, ff3_url: &str, ff3_token: &str) -> Arc<dyn FireflyClient> {
        Arc::new(ReqwestFireflyClient::new(ff3_url, ff3_token))
    }
}
