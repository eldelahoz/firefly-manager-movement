//! Unit tests for `GetAccountTransactions` cache enrichment.

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};

use ffm_application::dtos::{
    About, BrowseTxDto, Ff3AccountDto, Ff3TransactionDto, Ff3TxStatus, NewFf3TransactionCreation,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{Ff3TransactionRepository, FireflyClient, MatchRepository};
use ffm_application::use_cases::get_account_transactions::GetAccountTransactions;
use ffm_domain::entities::{Ff3Transaction, Match};
use ffm_domain::ids::{Ff3TransactionId, ReconciliationId};
use ffm_domain::value_objects::AmountCents;

// ── Fakes ───────────────────────────────────────────────────────────────

struct FakeClient {
    txs: Vec<Ff3TransactionDto>,
}

#[async_trait]
impl FireflyClient for FakeClient {
    async fn about(&self) -> Result<About, ApplicationError> {
        Ok(About {
            version: "6.1.36".into(),
        })
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: Option<NaiveDate>,
        _end: Option<NaiveDate>,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        Ok(self.txs.clone())
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
}

struct FakeTxCache {
    rows: Vec<Ff3Transaction>,
}

#[async_trait]
impl Ff3TransactionRepository for FakeTxCache {
    async fn insert_batch(
        &self,
        _transactions: Vec<ffm_application::dtos::NewFf3Transaction>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_reconciliation(
        &self,
        _reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        unimplemented!()
    }
    async fn latest_by_ff3_id(
        &self,
        _ff3_id: i64,
    ) -> Result<Option<Ff3Transaction>, ApplicationError> {
        unimplemented!()
    }
    async fn latest_by_ff3_ids(
        &self,
        ff3_ids: &[i64],
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        let mut latest = std::collections::HashMap::new();
        for row in &self.rows {
            if ff3_ids.contains(&row.ff3_id) {
                latest.insert(row.ff3_id, row.clone());
            }
        }
        Ok(latest.into_values().collect())
    }
}

struct FakeMatchRepo {
    matched: HashSet<i64>,
}

#[async_trait]
impl MatchRepository for FakeMatchRepo {
    async fn insert_batch(
        &self,
        _matches: Vec<ffm_application::dtos::NewMatch>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_reconciliation(
        &self,
        _reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Match>, ApplicationError> {
        unimplemented!()
    }
    async fn matched_ff3_ids(&self, ff3_ids: &[i64]) -> Result<HashSet<i64>, ApplicationError> {
        let wanted: HashSet<i64> = ff3_ids.iter().copied().collect();
        Ok(self.matched.intersection(&wanted).copied().collect())
    }
}

// ── Fixtures ────────────────────────────────────────────────────────────

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).expect("date")
}

fn epoch() -> NaiveDateTime {
    chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0)
        .expect("epoch")
        .naive_utc()
}

fn live(ff3_id: i64) -> Ff3TransactionDto {
    Ff3TransactionDto {
        ff3_id,
        date: d(2026, 8, 1),
        amount_cents: AmountCents::new(-100),
        description: format!("tx{ff3_id}"),
        splits: vec![],
    }
}

fn cached(local_id: i64, ff3_id: i64, dup: bool) -> Ff3Transaction {
    Ff3Transaction {
        ff3_journal_id: 0,
        id: Ff3TransactionId::new(local_id),
        reconciliation_id: ReconciliationId::new(local_id),
        ff3_id,
        date: d(2026, 8, 1),
        amount_cents: AmountCents::new(-100),
        description: format!("tx{ff3_id}"),
        is_duplicate: dup,
        created_at: epoch(),
    }
}

fn use_case(
    txs: Vec<Ff3TransactionDto>,
    cache: Vec<Ff3Transaction>,
    matched: HashSet<i64>,
) -> GetAccountTransactions<FakeClient, FakeTxCache, FakeMatchRepo> {
    GetAccountTransactions::new(
        FakeClient { txs },
        Arc::new(FakeTxCache { rows: cache }),
        Arc::new(FakeMatchRepo { matched }),
    )
}

fn status_of(txs: &[BrowseTxDto], ff3_id: i64) -> Option<Ff3TxStatus> {
    txs.iter().find(|t| t.ff3_id == ff3_id).unwrap().status
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn enriches_status_from_cache() {
    let uc = use_case(
        vec![live(1), live(2), live(3), live(4), live(5)],
        vec![
            cached(1, 1, false), // matched (has a match row)
            cached(2, 2, true),  // duplicate flag wins even with a match
            cached(3, 3, false), // cached without match → extra
            cached(4, 5, false), // older run: not duplicate…
            cached(5, 5, true),  // …latest run flags duplicate
        ],
        HashSet::from([1, 2]),
    );
    let txs = uc
        .execute(9, Some(d(2026, 8, 1)), Some(d(2026, 8, 31)))
        .await
        .unwrap();
    assert_eq!(txs.len(), 5);
    assert_eq!(status_of(&txs, 1), Some(Ff3TxStatus::Matched));
    assert_eq!(status_of(&txs, 2), Some(Ff3TxStatus::Duplicate));
    assert_eq!(status_of(&txs, 3), Some(Ff3TxStatus::Extra));
    assert_eq!(status_of(&txs, 4), None); // never seen in a reconciliation
    assert_eq!(status_of(&txs, 5), Some(Ff3TxStatus::Duplicate)); // latest row wins
}

#[tokio::test]
async fn keeps_validation_of_period() {
    let uc = use_case(vec![], vec![], HashSet::new());
    let err = uc
        .execute(9, Some(d(2026, 8, 31)), Some(d(2026, 8, 1)))
        .await;
    assert!(matches!(err, Err(ApplicationError::InvalidState(_))));
}
