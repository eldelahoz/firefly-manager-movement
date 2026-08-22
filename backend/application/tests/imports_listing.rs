use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{NewBatch, NewMovement};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{BatchRepository, MovementRepository};
use ffm_application::use_cases::get_import::GetImport;
use ffm_application::use_cases::list_imports::ListImports;

use ffm_domain::entities::{Batch, Movement};
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

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

    async fn update_status(&self, _: BatchId, _: BatchStatus) -> Result<(), ApplicationError> {
        Ok(())
    }

    async fn update_account(
        &self,
        _: BatchId,
        _: ffm_domain::ids::Ff3AccountId,
    ) -> Result<(), ApplicationError> {
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
        _: MovementId,
        _: MovementStatus,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[tokio::test]
async fn list_imports_returns_batches() {
    let batches = Arc::new(FakeBatchRepo::new());
    let uc = ListImports::new(batches.clone());

    batches
        .insert(NewBatch {
            filename: "a.csv".into(),
            file_hash: "h1".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .unwrap();
    batches
        .insert(NewBatch {
            filename: "b.csv".into(),
            file_hash: "h2".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .unwrap();

    let list = uc.execute().await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].filename, "a.csv");
    assert_eq!(list[1].filename, "b.csv");
}

#[tokio::test]
async fn get_import_returns_batch_and_movements() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let uc = GetImport::new(batches.clone(), movements.clone());

    let batch_id = batches
        .insert(NewBatch {
            filename: "a.csv".into(),
            file_hash: "h1".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
        })
        .await
        .unwrap();

    movements
        .insert_batch(vec![
            NewMovement {
                batch_id,
                line_no: 1,
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                amount_cents: AmountCents::new(10000),
                description: "Coffee".into(),
                raw_json: "{}".into(),
                status: MovementStatus::Imported,
            },
            NewMovement {
                batch_id,
                line_no: 2,
                date: NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
                amount_cents: AmountCents::new(-5000),
                description: "Lunch".into(),
                raw_json: "{}".into(),
                status: MovementStatus::Imported,
            },
        ])
        .await
        .unwrap();

    let (batch, mvs) = uc.execute(batch_id).await.unwrap();
    assert_eq!(batch.id, batch_id);
    assert_eq!(batch.filename, "a.csv");
    assert_eq!(mvs.len(), 2);
    assert_eq!(mvs[0].line_no, 1);
    assert_eq!(mvs[1].line_no, 2);
}
