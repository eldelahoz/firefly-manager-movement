//! GetAccountTransactions use case.
//! Fetches transactions for a Firefly III account within a date range
//! and enriches each one with its reconciliation status derived from
//! the local run cache (browse widget).

use std::collections::HashSet;
use std::sync::Arc;

use chrono::NaiveDate;

use crate::dtos::{BrowseTxDto, Ff3TransactionDto, Ff3TxStatus};
use crate::error::ApplicationError;
use crate::ports::{Ff3TransactionRepository, FireflyClient, MatchRepository};

pub struct GetAccountTransactions<C, T: ?Sized, M: ?Sized> {
    client: C,
    ff3_transactions: Arc<T>,
    matches: Arc<M>,
}

impl<C, T: ?Sized, M: ?Sized> GetAccountTransactions<C, T, M> {
    pub fn new(client: C, ff3_transactions: Arc<T>, matches: Arc<M>) -> Self {
        Self {
            client,
            ff3_transactions,
            matches,
        }
    }
}

impl<C: FireflyClient, T: Ff3TransactionRepository + ?Sized, M: MatchRepository + ?Sized>
    GetAccountTransactions<C, T, M>
{
    pub async fn execute(
        &self,
        account_ff3_id: i64,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> Result<Vec<BrowseTxDto>, ApplicationError> {
        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return Err(ApplicationError::InvalidState(
                    "start date must not be after end date".into(),
                ));
            }
        }
        if start.is_none() && end.is_none() {
            return Err(ApplicationError::InvalidState(
                "at least one of start or end must be provided".into(),
            ));
        }
        let txs = self
            .client
            .account_transactions(account_ff3_id, start, end)
            .await?;
        let ids: Vec<i64> = txs.iter().map(|t| t.ff3_id).collect();
        let cached = self.ff3_transactions.latest_by_ff3_ids(&ids).await?;
        let matched = self.matches.matched_ff3_ids(&ids).await?;
        Ok(txs
            .into_iter()
            .map(|t| {
                let status = enrich_status(&t, &cached, &matched);
                BrowseTxDto {
                    ff3_id: t.ff3_id,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description,
                    status,
                }
            })
            .collect())
    }
}

fn enrich_status(
    tx: &Ff3TransactionDto,
    cached: &[ffm_domain::entities::Ff3Transaction],
    matched: &HashSet<i64>,
) -> Option<Ff3TxStatus> {
    let latest = cached.iter().find(|c| c.ff3_id == tx.ff3_id)?;
    if latest.is_duplicate {
        Some(Ff3TxStatus::Duplicate)
    } else if matched.contains(&tx.ff3_id) {
        Some(Ff3TxStatus::Matched)
    } else {
        Some(Ff3TxStatus::Extra)
    }
}
