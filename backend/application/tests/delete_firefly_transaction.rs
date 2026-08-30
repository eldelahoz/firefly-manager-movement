use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewDeletedAudit, NewFf3Transaction,
    NewFf3TransactionCreation,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{DeletedAuditRepository, Ff3TransactionRepository, FireflyClient};
use ffm_application::use_cases::delete_firefly_transaction::DeleteFireflyTransaction;

use ffm_domain::entities::Ff3Transaction;
use ffm_domain::ids::{DeletedAuditId, ReconciliationId};
use ffm_domain::value_objects::AmountCents;

// ── Fakes ───────────────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

struct FakeFirefly {
    deleted: Mutex<Vec<i64>>,
    fail: bool,
}

#[async_trait]
impl FireflyClient for FakeFirefly {
    async fn about(&self) -> Result<About, ApplicationError> {
        unimplemented!()
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        unimplemented!()
    }
    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        unimplemented!()
    }
    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        unimplemented!()
    }
    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        if self.fail {
            return Err(ApplicationError::FireflyClient("FF3 500".into()));
        }
        self.deleted.lock().unwrap().push(ff3_id);
        Ok(())
    }
}

struct FakeFf3TxRepo {
    rows: Vec<Ff3Transaction>,
}

#[async_trait]
impl Ff3TransactionRepository for FakeFf3TxRepo {
    async fn insert_batch(
        &self,
        _transactions: Vec<NewFf3Transaction>,
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
        ff3_id: i64,
    ) -> Result<Option<Ff3Transaction>, ApplicationError> {
        // Later rows (higher local id) win — "latest" cached copy.
        Ok(self
            .rows
            .iter()
            .filter(|t| t.ff3_id == ff3_id)
            .max_by_key(|t| t.id.as_i64())
            .cloned())
    }
    async fn latest_by_ff3_ids(
        &self,
        _ff3_ids: &[i64],
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        unimplemented!()
    }
}

#[derive(Default)]
struct FakeDeletedAuditRepo {
    rows: Mutex<Vec<NewDeletedAudit>>,
}

#[async_trait]
impl DeletedAuditRepository for FakeDeletedAuditRepo {
    async fn insert(&self, audit: NewDeletedAudit) -> Result<DeletedAuditId, ApplicationError> {
        self.rows.lock().unwrap().push(audit);
        Ok(DeletedAuditId::new(self.rows.lock().unwrap().len() as i64))
    }
}

// ── Fixture ─────────────────────────────────────────────────────────────

fn cached_tx(local_id: i64, ff3_id: i64, dup: bool) -> Ff3Transaction {
    Ff3Transaction {
        id: ffm_domain::ids::Ff3TransactionId::new(local_id),
        reconciliation_id: ReconciliationId::new(10),
        ff3_id,
        date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        amount_cents: AmountCents::new(-4200),
        description: "dupe".into(),
        is_duplicate: dup,
        created_at: epoch(),
    }
}

struct Fx {
    firefly: Arc<FakeFirefly>,
    txs: Arc<FakeFf3TxRepo>,
    audits: Arc<FakeDeletedAuditRepo>,
}

fn fx(fail_delete: bool, rows: Vec<Ff3Transaction>) -> Fx {
    Fx {
        firefly: Arc::new(FakeFirefly {
            deleted: Mutex::new(vec![]),
            fail: fail_delete,
        }),
        txs: Arc::new(FakeFf3TxRepo { rows }),
        audits: Arc::new(FakeDeletedAuditRepo::default()),
    }
}

impl Fx {
    fn uc(
        &self,
    ) -> DeleteFireflyTransaction<
        Arc<dyn FireflyClient>,
        Arc<FakeFf3TxRepo>,
        Arc<FakeDeletedAuditRepo>,
    > {
        DeleteFireflyTransaction::new(
            self.firefly.clone() as Arc<dyn FireflyClient>,
            self.txs.clone(),
            self.audits.clone(),
        )
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_snapshots_then_deletes_then_audits_rf15() {
    let f = fx(
        false,
        vec![cached_tx(1, 300, false), cached_tx(2, 999, true)],
    );

    f.uc().execute(999).await.expect("delete ok");

    // FF3 called with the external id.
    assert_eq!(*f.firefly.deleted.lock().unwrap(), vec![999]);

    // Audit row: ff3_id + point-in-time snapshot of the cached row.
    let rows = f.audits.rows.lock().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ff3_id, 999);
    let snapshot: serde_json::Value = serde_json::from_str(&rows[0].snapshot_json).unwrap();
    assert_eq!(snapshot["ff3_id"], 999);
    assert_eq!(snapshot["amount_cents"], -4200);
    assert_eq!(snapshot["description"], "dupe");
    assert_eq!(snapshot["is_duplicate"], true);
    assert_eq!(snapshot["date"], "2026-01-15");
}

#[tokio::test]
async fn delete_unknown_ff3_transaction_is_not_found() {
    let f = fx(false, vec![cached_tx(1, 300, false)]);
    let err = f.uc().execute(4242).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
    assert!(f.audits.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_ff3_failure_leaves_no_audit() {
    let f = fx(true, vec![cached_tx(1, 999, true)]);
    let err = f.uc().execute(999).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::FireflyClient(_)));
    assert!(f.audits.rows.lock().unwrap().is_empty());
}
