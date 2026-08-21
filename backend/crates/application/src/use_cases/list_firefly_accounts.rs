//! ListFireflyAccounts use case.
//! Fetches asset accounts from Firefly III and syncs them to the local cache.

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
    pub async fn execute(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        let dtos = self.client.asset_accounts().await?;
        let new_accounts: Vec<NewFf3Account> = dtos
            .iter()
            .map(|d| NewFf3Account {
                ff3_id: d.ff3_id,
                name: d.name.clone(),
                account_type: d.account_type.clone(),
                currency: d.currency.clone(),
            })
            .collect();
        self.accounts.upsert_batch(new_accounts).await?;
        Ok(dtos)
    }
}
