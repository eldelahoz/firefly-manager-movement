//! ListFireflyAccounts use case.
//! Fetches accounts from Firefly III by type and syncs them to the local cache.

use crate::dtos::{Ff3AccountDto, NewFf3Account};
use crate::error::ApplicationError;
use crate::ports::{Ff3AccountRepository, FireflyClient};

pub struct ListFireflyAccounts<C, R> {
    client: C,
    accounts: R,
}

impl<C, R> ListFireflyAccounts<C, R> {
    pub fn new(client: C, accounts: R) -> Self {
        Self { client, accounts }
    }
}

impl<C: FireflyClient, R: Ff3AccountRepository> ListFireflyAccounts<C, R> {
    pub async fn execute(&self, account_type: &str) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        let dtos = self.client.accounts_by_type(account_type).await?;
        let new_accounts: Vec<NewFf3Account> = dtos
            .iter()
            .map(|d| NewFf3Account {
                ff3_id: d.ff3_id,
                name: d.name.clone(),
                account_type: d.account_type.clone(),
                currency: d.currency.clone(),
                iban: d.iban.clone(),
                account_number: d.account_number.clone(),
            })
            .collect();
        self.accounts.upsert_batch(new_accounts).await?;
        Ok(dtos)
    }
}
