//! Shared test fakes for presentation E2E tests.

use async_trait::async_trait;
use chrono::NaiveDate;

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewFf3Account, NewFf3TransactionCreation,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{Ff3AccountRepository, FireflyClient};
use ffm_domain::entities::Ff3Account;

// ── FakeFireflyClient ──────────────────────────────────────────────────

pub struct FakeFireflyClient {
    pub fail: bool,
    pub accounts: Vec<Ff3AccountDto>,
    pub transactions: Vec<Ff3TransactionDto>,
}

impl FakeFireflyClient {
    #[allow(dead_code)]
    pub fn ok() -> Self {
        Self {
            fail: false,
            accounts: vec![],
            transactions: vec![],
        }
    }

    pub fn failing() -> Self {
        Self {
            fail: true,
            accounts: vec![],
            transactions: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn with_accounts(accounts: Vec<Ff3AccountDto>) -> Self {
        Self {
            fail: false,
            accounts,
            transactions: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn with_transactions(transactions: Vec<Ff3TransactionDto>) -> Self {
        Self {
            fail: false,
            accounts: vec![],
            transactions,
        }
    }
}

#[async_trait]
impl FireflyClient for FakeFireflyClient {
    async fn about(&self) -> Result<About, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(About {
                version: "6.1.36".into(),
            })
        }
    }

    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(self.accounts.clone())
        }
    }

    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(self.transactions.clone())
        }
    }

    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(100)
        }
    }

    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(())
        }
    }
}

// ── FakeFf3AccountRepository ───────────────────────────────────────────

pub struct FakeFf3AccountRepository {
    upserted: std::sync::Mutex<Vec<NewFf3Account>>,
}

impl FakeFf3AccountRepository {
    pub fn new() -> Self {
        Self {
            upserted: std::sync::Mutex::new(vec![]),
        }
    }

    #[allow(dead_code)]
    pub fn upserted_count(&self) -> usize {
        self.upserted.lock().unwrap().len()
    }
}

impl Default for FakeFf3AccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Ff3AccountRepository for FakeFf3AccountRepository {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        self.upserted.lock().unwrap().extend(accounts);
        Ok(())
    }

    async fn by_ff3_id(&self, _ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok(None)
    }

    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        Ok(vec![])
    }
}
