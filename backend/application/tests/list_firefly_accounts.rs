//! `ListFireflyAccounts` and the `Arc<dyn FireflyClient>` wrapper the handlers use.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewFf3Account, NewFf3TransactionCreation,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{Ff3AccountRepository, FireflyClient};
use ffm_application::use_cases::list_firefly_accounts::ListFireflyAccounts;
use ffm_domain::entities::Ff3Account;
use ffm_domain::ids::Ff3AccountId;

/// Behaves like Firefly III: `asset_accounts` only serves assets, other types
/// come from `accounts_by_type`.
struct TypedFirefly;

fn groceries() -> Ff3AccountDto {
    Ff3AccountDto {
        ff3_id: 7,
        name: "Groceries".into(),
        account_type: "expense".into(),
        currency: "EUR".into(),
        iban: None,
        account_number: None,
    }
}

#[async_trait]
impl FireflyClient for TypedFirefly {
    async fn about(&self) -> Result<About, ApplicationError> {
        unimplemented!()
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn accounts_by_type(
        &self,
        account_type: &str,
    ) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(if account_type == "expense" {
            vec![groceries()]
        } else {
            vec![]
        })
    }
    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: Option<NaiveDate>,
        _end: Option<NaiveDate>,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        unimplemented!()
    }
    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        unimplemented!()
    }
    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn tags(&self) -> Result<Vec<String>, ApplicationError> {
        Ok(vec!["tag".into()])
    }
    async fn categories(&self) -> Result<Vec<String>, ApplicationError> {
        Ok(vec!["category".into()])
    }
    async fn budgets(&self) -> Result<Vec<String>, ApplicationError> {
        Ok(vec!["budget".into()])
    }
}

struct NoopAccounts;

#[async_trait]
impl Ff3AccountRepository for NoopAccounts {
    async fn upsert_batch(&self, _accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn by_ff3_id(&self, _ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        unimplemented!()
    }
    async fn by_id(&self, _id: Ff3AccountId) -> Result<Option<Ff3Account>, ApplicationError> {
        unimplemented!()
    }
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        unimplemented!()
    }
}

fn wrapped() -> Arc<dyn FireflyClient> {
    Arc::new(TypedFirefly)
}

#[tokio::test]
async fn lists_expense_accounts_through_the_arc_wrapper() {
    let uc = ListFireflyAccounts::new(wrapped(), NoopAccounts);

    let accounts = uc.execute("expense").await.unwrap();

    let ids: Vec<(i64, &str)> = accounts
        .iter()
        .map(|a| (a.ff3_id, a.account_type.as_str()))
        .collect();
    assert_eq!(ids, vec![(7, "expense")]);
}

#[tokio::test]
async fn arc_wrapper_delegates_tags_categories_and_budgets() {
    let client = wrapped();

    assert_eq!(FireflyClient::tags(&client).await.unwrap(), vec!["tag"]);
    assert_eq!(
        FireflyClient::categories(&client).await.unwrap(),
        vec!["category"]
    );
    assert_eq!(
        FireflyClient::budgets(&client).await.unwrap(),
        vec!["budget"]
    );
}
