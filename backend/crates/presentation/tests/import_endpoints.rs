use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use chrono::{DateTime, NaiveDateTime, Utc};
use ffm_application::dtos::{NewBatch, NewCsvMapping, NewMovement};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, CsvDecoder, CsvMappingRepository, MovementRepository,
};
use ffm_domain::entities::{Batch, CsvMapping, Movement};
use ffm_domain::ids::{BatchId, CsvMappingId, MovementId};
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};
use ffm_presentation::{app, AppState};
use tower::ServiceExt;

mod common;
use common::{FakeFf3AccountRepository, FakeFireflyClient};

// ── Fakes for import stack ─────────────────────────────────────────────

struct FakeCsvDecoder;

#[async_trait]
impl CsvDecoder for FakeCsvDecoder {
    async fn decode(&self, bytes: Vec<u8>) -> Result<String, ApplicationError> {
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

struct FakeBatchRepo {
    batches: Mutex<Vec<Batch>>,
    next_id: Mutex<i64>,
}

impl FakeBatchRepo {
    fn new() -> Self {
        Self {
            batches: Mutex::new(vec![]),
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
            created_at: epoch(),
        });
        Ok(batch_id)
    }
    async fn get(&self, _id: BatchId) -> Result<Batch, ApplicationError> {
        let id = _id;
        self.batches
            .lock()
            .unwrap()
            .iter()
            .find(|b| b.id == id)
            .cloned()
            .ok_or_else(|| ApplicationError::NotFound("batch".into()))
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        Ok(self.batches.lock().unwrap().clone())
    }
    async fn update_status(
        &self,
        _id: BatchId,
        _status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn update_account(
        &self,
        _id: BatchId,
        _account_id: ffm_domain::ids::Ff3AccountId,
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
            movements: Mutex::new(vec![]),
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
    async fn by_batch(&self, _batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        Ok(self
            .movements
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.batch_id == _batch_id)
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
}

struct FakeCsvMappingRepo {
    mappings: Mutex<Vec<CsvMapping>>,
}

impl FakeCsvMappingRepo {
    fn new() -> Self {
        Self {
            mappings: Mutex::new(vec![]),
        }
    }
}

#[async_trait]
impl CsvMappingRepository for FakeCsvMappingRepo {
    async fn by_signature(
        &self,
        _signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError> {
        Ok(self
            .mappings
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.header_signature.as_str() == _signature.as_str())
            .cloned())
    }
    async fn upsert(&self, _mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError> {
        let mut store = self.mappings.lock().unwrap();
        if let Some(existing) = store
            .iter_mut()
            .find(|m| m.header_signature.as_str() == _mapping.header_signature.as_str())
        {
            existing.mapping_json = _mapping.mapping_json;
            return Ok(existing.id);
        }
        let id = CsvMappingId::new(store.len() as i64 + 1);
        store.push(CsvMapping {
            id,
            header_signature: _mapping.header_signature,
            mapping_json: _mapping.mapping_json,
            created_at: epoch(),
        });
        Ok(id)
    }
}

#[tokio::test]
async fn post_imports_parses_and_persists_movements() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let mappings = Arc::new(FakeCsvMappingRepo::new());

    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(FakeCsvDecoder),
        batches: batches.clone(),
        movements: movements.clone(),
        csv_mappings: mappings.clone(),
        ff3_health_probe: false,
    };
    let router = app(state);

    let boundary = "BOUNDARY";
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv}\r\n--{b}\r\nContent-Disposition: form-data; name=\"currency\"\r\n\r\nEUR\r\n--{b}--\r\n",
        b = boundary,
        csv = csv
    );

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/imports")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["filename"], "test.csv");
    assert_eq!(value["headers"].as_array().unwrap().len(), 3);
    assert_eq!(value["total_rows"], 1);
    assert_eq!(value["valid_count"], 1);
    assert_eq!(value["invalid_count"], 0);
    assert_eq!(value["valid"].as_array().unwrap().len(), 1);
    assert_eq!(value["valid"][0]["amount_cents"], 10000);

    // persisted
    assert_eq!(batches.batches.lock().unwrap().len(), 1);
    assert_eq!(movements.movements.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn get_imports_lists_batches() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let mappings = Arc::new(FakeCsvMappingRepo::new());

    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(FakeCsvDecoder),
        batches: batches.clone(),
        movements: movements.clone(),
        csv_mappings: mappings,
        ff3_health_probe: false,
    };
    let router = app(state);

    // Create one batch via upload.
    let boundary = "BOUNDARY";
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv}\r\n--{b}--\r\n",
        b = boundary,
        csv = csv
    );

    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/imports")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/imports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["filename"], "test.csv");
}

#[tokio::test]
async fn get_import_returns_batch_and_movements() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let mappings = Arc::new(FakeCsvMappingRepo::new());

    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(FakeCsvDecoder),
        batches: batches.clone(),
        movements: movements.clone(),
        csv_mappings: mappings,
        ff3_health_probe: false,
    };
    let router = app(state);

    let boundary = "BOUNDARY";
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n2026-01-06,-50.00,Lunch\n";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv}\r\n--{b}--\r\n",
        b = boundary,
        csv = csv
    );

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/imports")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/imports/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["batch"]["filename"], "test.csv");
    assert_eq!(value["movements"].as_array().unwrap().len(), 2);
    assert_eq!(value["movements"][0]["amount_cents"], 10000);
    assert_eq!(value["movements"][1]["amount_cents"], -5000);
}

#[tokio::test]
async fn put_import_mapping_saves_mapping() {
    let batches = Arc::new(FakeBatchRepo::new());
    let movements = Arc::new(FakeMovementRepo::new());
    let mappings = Arc::new(FakeCsvMappingRepo::new());

    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(FakeCsvDecoder),
        batches: batches.clone(),
        movements: movements.clone(),
        csv_mappings: mappings.clone(),
        ff3_health_probe: false,
    };
    let router = app(state);

    // Create the batch via upload.
    let boundary = "BOUNDARY";
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv}\r\n--{b}--\r\n",
        b = boundary,
        csv = csv
    );
    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/imports")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = serde_json::json!({
        "headers": ["Date", "Amount", "Description"],
        "date_column": "Date",
        "amount_column": "Amount",
        "description_column": "Description"
    });
    let response = router
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/imports/1/mapping")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["mapping_id"], 1);

    // Stored by signature.
    let sig = HeaderSignature::from_headers(&[
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
    ]);
    let stored = mappings.by_signature(sig).await.unwrap().unwrap();
    assert!(stored.mapping_json.contains("date_column"));
}
