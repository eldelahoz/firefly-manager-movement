use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};

use ffm_application::dtos::{NewBatch, NewMovement};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{BatchRepository, MovementRepository};
use ffm_application::use_cases::delete_import::DeleteImport;

use ffm_domain::entities::{Batch, Movement};
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};

// ── Fakes ───────────────────────────────────────────────────────────────

struct FakeBatchRepo {
    batches: Mutex<Vec<Batch>>,
    next_id: Mutex<i64>,
}

impl FakeBatchRepo {
    fn new() -> Self {
        Self {
            batches: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

#[async_trait]
impl BatchRepository for FakeBatchRepo {
    async fn insert(&self, batch: NewBatch) -> Result<BatchId, ApplicationError> {
        let mut id = self.next_id.lock().unwrap();
        let batch_id = BatchId::new(*id);
        *id += 1;
        self.batches.lock().unwrap().push(Batch {
            id: batch_id,
            filename: batch.filename,
            file_hash: batch.file_hash,
            format: batch.format,
            currency: batch.currency,
            account_id: batch.account_id,
            status: batch.status,
            headers: batch.headers,
            detected_delimiter: batch.detected_delimiter,
            created_at: epoch(),
        });
        Ok(batch_id)
    }
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError> {
        self.batches
            .lock()
            .unwrap()
            .iter()
            .find(|b| b.id == id)
            .cloned()
            .ok_or_else(|| ApplicationError::NotFound("batch not found".into()))
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        Ok(self.batches.lock().unwrap().clone())
    }
    async fn update_status(
        &self,
        id: BatchId,
        status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        if let Some(b) = self.batches.lock().unwrap().iter_mut().find(|b| b.id == id) {
            b.status = status;
        }
        Ok(())
    }
    async fn update_account(
        &self,
        _id: BatchId,
        _account_id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn delete(&self, id: BatchId) -> Result<(), ApplicationError> {
        self.batches.lock().unwrap().retain(|b| b.id != id);
        Ok(())
    }
}

struct FakeMovementRepo {
    movements: Mutex<Vec<Movement>>,
}

impl FakeMovementRepo {
    fn new() -> Self {
        Self {
            movements: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl MovementRepository for FakeMovementRepo {
    async fn insert_batch(&self, movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        let mut store = self.movements.lock().unwrap();
        for m in movements {
            let next_id = store.len() as i64 + 1;
            store.push(Movement {
                id: MovementId::new(next_id),
                batch_id: m.batch_id,
                line_no: m.line_no,
                date: m.date,
                amount_cents: m.amount_cents,
                description: m.description,
                raw_json: m.raw_json,
                status: m.status,
                created_at: epoch(),
            });
        }
        Ok(())
    }
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        Ok(self
            .movements
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.batch_id == batch_id)
            .cloned()
            .collect())
    }
    async fn update_status(
        &self,
        _id: MovementId,
        _status: MovementStatus,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn delete_by_batch(&self, batch_id: BatchId) -> Result<(), ApplicationError> {
        self.movements
            .lock()
            .unwrap()
            .retain(|m| m.batch_id != batch_id);
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

fn sample_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 1, 5).expect("valid date")
}

async fn seed_parsed_batch(
    batches: &Arc<FakeBatchRepo>,
    movements: &Arc<FakeMovementRepo>,
) -> BatchId {
    let id = batches
        .insert(NewBatch {
            filename: "test.csv".into(),
            file_hash: "hash".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
            headers: vec![],
            detected_delimiter: ",".into(),
        })
        .await
        .unwrap();
    movements
        .insert_batch(vec![NewMovement {
            batch_id: id,
            line_no: 1,
            date: sample_date(),
            amount_cents: AmountCents::new(10000),
            description: "Coffee".into(),
            raw_json: serde_json::json!({}).to_string(),
            status: MovementStatus::Imported,
        }])
        .await
        .unwrap();
    id
}

#[tokio::test]
async fn delete_import_removes_parsed_batch_and_movements() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let id = seed_parsed_batch(&batches, &movements).await;

    DeleteImport::new(batches.clone(), movements.clone())
        .execute(id)
        .await
        .unwrap();

    assert!(batches.batches.lock().unwrap().is_empty());
    assert!(movements.movements.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_import_rejects_non_parsed_batch() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let id = seed_parsed_batch(&batches, &movements).await;
    batches
        .update_status(id, BatchStatus::Reconciled)
        .await
        .unwrap();

    let err = DeleteImport::new(batches.clone(), movements.clone())
        .execute(id)
        .await
        .unwrap_err();

    assert!(matches!(err, ApplicationError::InvalidState(_)));
    assert_eq!(batches.batches.lock().unwrap().len(), 1);
    assert_eq!(movements.movements.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn delete_import_missing_batch_is_not_found() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());

    let err = DeleteImport::new(batches, movements)
        .execute(BatchId::new(999))
        .await
        .unwrap_err();

    assert!(matches!(err, ApplicationError::NotFound(_)));
}
