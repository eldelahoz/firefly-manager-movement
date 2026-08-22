//! GetAccountTransactions use case.
//! Fetches transactions for a Firefly III account within a date range.

use chrono::NaiveDate;

use crate::dtos::Ff3TransactionDto;
use crate::error::ApplicationError;
use crate::ports::FireflyClient;

pub struct GetAccountTransactions<C> {
    client: C,
}

impl<C> GetAccountTransactions<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C: FireflyClient> GetAccountTransactions<C> {
    pub async fn execute(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        if start > end {
            return Err(ApplicationError::InvalidState(
                "start date must not be after end date".into(),
            ));
        }
        self.client
            .account_transactions(account_ff3_id, start, end)
            .await
    }
}
