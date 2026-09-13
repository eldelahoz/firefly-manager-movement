use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};

use ffm_application::csv::ColumnMapping;
use ffm_application::dtos::{NewBatch, NewCsvMapping, NewMovement};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, CsvDecoder, CsvMappingRepository, MovementRepository,
};
use ffm_application::use_cases::upload_import::{UploadImport, UploadImportInput};

use ffm_domain::entities::{Batch, CsvMapping, Movement};
use ffm_domain::ids::{BatchId, CsvMappingId, MovementId};
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};

// ── Fakes ───────────────────────────────────────────────────────────────

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
        id: BatchId,
        account_id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        if let Some(b) = self.batches.lock().unwrap().iter_mut().find(|b| b.id == id) {
            b.account_id = Some(account_id);
        }
        Ok(())
    }
    async fn find_by_filename(&self, filename: &str) -> Result<Vec<Batch>, ApplicationError> {
        Ok(self
            .batches
            .lock()
            .unwrap()
            .iter()
            .filter(|b| b.filename == filename)
            .cloned()
            .collect())
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
        id: MovementId,
        status: MovementStatus,
    ) -> Result<(), ApplicationError> {
        if let Some(m) = self
            .movements
            .lock()
            .unwrap()
            .iter_mut()
            .find(|m| m.id == id)
        {
            m.status = status;
        }
        Ok(())
    }
}

struct FakeCsvMappingRepo {
    mappings: Mutex<Vec<CsvMapping>>,
}

impl FakeCsvMappingRepo {
    fn new() -> Self {
        Self {
            mappings: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl CsvMappingRepository for FakeCsvMappingRepo {
    async fn by_signature(
        &self,
        signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError> {
        Ok(self
            .mappings
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.header_signature.as_str() == signature.as_str())
            .cloned())
    }
    async fn upsert(&self, mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError> {
        let mut store = self.mappings.lock().unwrap();
        if let Some(existing) = store
            .iter_mut()
            .find(|m| m.header_signature.as_str() == mapping.header_signature.as_str())
        {
            existing.mapping_json = mapping.mapping_json;
            return Ok(existing.id);
        }
        let id = CsvMappingId::new(store.len() as i64 + 1);
        store.push(CsvMapping {
            id,
            header_signature: mapping.header_signature,
            mapping_json: mapping.mapping_json,
            created_at: epoch(),
        });
        Ok(id)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn upload_import_parses_valid_csv() {
    let csv =
        b"Date,Amount,Description\n2026-01-05,100.00,Coffee\n2026-01-06,-50.00,Lunch".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    assert_eq!(result.filename, "test.csv");
    assert_eq!(result.total_rows, 2);
    assert_eq!(result.valid_count, 2);
    assert_eq!(result.invalid_count, 0);
    assert_eq!(result.headers, vec!["Date", "Amount", "Description"]);
    assert_eq!(result.valid.len(), 2);
    assert!(result.invalid.is_empty());
    assert!(result.mapping_used.is_some());

    let movements = movement_repo.movements.lock().unwrap();
    assert_eq!(movements.len(), 2);
    assert_eq!(movements[0].line_no, 1);
    assert_eq!(movements[0].amount_cents.as_i64(), 10000);
    assert_eq!(movements[1].line_no, 2);
    assert_eq!(movements[1].amount_cents.as_i64(), -5000);
}

#[tokio::test]
async fn upload_import_keeps_invalid_lines() {
    let csv = b"Date,Amount,Description\nbad-date,100.00,Coffee\n2026-01-06,abc,Lunch".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    assert_eq!(result.total_rows, 2);
    assert_eq!(result.valid_count, 0);
    assert_eq!(result.invalid_count, 2);
    assert_eq!(result.valid.len(), 0);
    assert_eq!(result.invalid.len(), 2);
    assert!(result.invalid[0].reason.contains("date"));
    assert!(result.invalid[1].reason.contains("amount"));
}

#[tokio::test]
async fn upload_import_uses_provided_mapping() {
    let csv = b"D,M,Desc\n2026-01-05,100.00,Coffee".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: Some(ColumnMapping {
                date_column: "D".into(),
                amount_column: "M".into(),
                description_column: "Desc".into(),
                origin_account_column: None,
            }),
        })
        .await
        .unwrap();

    assert_eq!(result.valid.len(), 1);
    assert_eq!(result.valid[0].amount_cents.as_i64(), 10000);
}

#[tokio::test]
async fn upload_import_no_mapping_returns_headers_only() {
    let csv = b"col1,col2,col3\nval1,val2,val3".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    assert!(result.mapping_used.is_none());
    assert_eq!(result.total_rows, 1);
    assert_eq!(result.valid_count, 0);
    assert_eq!(result.invalid_count, 0);
    assert!(result.valid.is_empty());
    assert!(result.invalid.is_empty());
    assert_eq!(result.headers, vec!["col1", "col2", "col3"]);

    // batch still created
    let batches = batch_repo.batches.lock().unwrap();
    assert_eq!(batches.len(), 1);
    // no movements inserted
    let movements = movement_repo.movements.lock().unwrap();
    assert!(movements.is_empty());
}

#[tokio::test]
async fn upload_import_persists_headers_and_delimiter_on_batch() {
    // semicolon-separated unknown headers: no mapping, headers-only probe
    let csv = b"col1;col2;col3\nv1;v2;v3".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "persist.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    let stored = batch_repo.get(result.batch_id).await.unwrap();
    assert_eq!(stored.headers, vec!["col1", "col2", "col3"]);
    assert_eq!(stored.detected_delimiter, ";");
}

#[tokio::test]
async fn upload_import_uses_saved_mapping_by_signature() {
    let csv = "Date,Montant,Libell\u{00e9}\n2026-01-05,100.00,Caf\u{00e9}"
        .as_bytes()
        .to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    // Pre-save a mapping for the header signature "date|montant|libellé"
    let signature = HeaderSignature::from_headers(&[
        "Date".to_string(),
        "Montant".to_string(),
        "Libellé".to_string(),
    ]);
    mapping_repo
        .upsert(NewCsvMapping {
            header_signature: signature,
            mapping_json: serde_json::json!({
                "date_column": "Date",
                "amount_column": "Montant",
                "description_column": "Libellé"
            })
            .to_string(),
        })
        .await
        .unwrap();

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    assert!(result.mapping_used.is_some());
    assert_eq!(result.valid.len(), 1);
    assert_eq!(result.valid[0].description, "Caf\u{00e9}");
}

#[tokio::test]
async fn upload_import_computes_file_hash() {
    let csv = b"Date,Amount,Description\n2026-01-05,100.00,Coffee".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv.clone(),
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    // SHA-256 produces 64 hex chars
    assert_eq!(result.file_hash.len(), 64);
    assert!(!result.file_hash.is_empty());

    // Same content -> same hash
    let result2 = uc
        .execute(UploadImportInput {
            filename: "test2.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();
    assert_eq!(result.file_hash, result2.file_hash);
}

#[tokio::test]
async fn upload_import_rejects_duplicate_filename() {
    let csv = b"Date,Amount,Description\n2026-01-05,100.00,Coffee".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    uc.execute(UploadImportInput {
        filename: "test.csv".into(),
        bytes: csv.clone(),
        currency: "EUR".into(),
        mapping: None,
    })
    .await
    .unwrap();

    let second = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await;

    match second {
        Err(ApplicationError::Conflict(_)) => {}
        Err(e) => panic!("expected Conflict, got other error: {e}"),
        Ok(_) => panic!("expected Conflict, got Ok"),
    }

    // Un filename distinto sí pasa.
    uc.execute(UploadImportInput {
        filename: "other.csv".into(),
        bytes: b"Date,Amount,Description\n2026-01-06,-1.00,Tea".to_vec(),
        currency: "EUR".into(),
        mapping: None,
    })
    .await
    .unwrap();

    let batches = batch_repo.batches.lock().unwrap();
    assert_eq!(batches.len(), 2);
}

#[tokio::test]
async fn upload_import_auto_detects_spanish_headers_valor() {
    let csv = "Fecha;Descripción;Referencia;Valor\n03/08/2026;Test;REF;-12,34\n"
        .as_bytes()
        .to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "movs.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: None,
        })
        .await
        .unwrap();

    assert!(result.mapping_used.is_some());
    assert_eq!(result.total_rows, 1);
    assert_eq!(result.valid_count, 1);
    assert_eq!(result.invalid_count, 0);
    assert_eq!(result.detected_delimiter, ';');
    assert_eq!(result.valid.len(), 1);
    assert_eq!(result.valid[0].amount_cents.as_i64(), -1234);
    assert_eq!(result.valid[0].description, "Test");
}

#[tokio::test]
async fn upload_import_extracts_origin_account_values() {
    let csv = b"Date,Amount,Description,Account\n2026-01-05,100.00,Coffee,ACC-001\n2026-01-06,-50.00,Lunch,ACC-002\n2026-01-07,25.00,Tea,ACC-001".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: Some(ColumnMapping {
                date_column: "Date".into(),
                amount_column: "Amount".into(),
                description_column: "Description".into(),
                origin_account_column: Some("Account".into()),
            }),
        })
        .await
        .unwrap();

    assert_eq!(result.origin_account_values, vec!["ACC-001", "ACC-002"]);
    assert_eq!(result.valid.len(), 3);
    assert_eq!(result.valid[0].origin_account.as_deref(), Some("ACC-001"));
    assert_eq!(result.valid[1].origin_account.as_deref(), Some("ACC-002"));
}

#[tokio::test]
async fn upload_import_empty_origin_account_values_when_no_column() {
    let csv = b"Date,Amount,Description\n2026-01-05,100.00,Coffee".to_vec();
    let decoder = FakeCsvDecoder;
    let batch_repo = Arc::new(FakeBatchRepo::new());
    let movement_repo = Arc::new(FakeMovementRepo::new());
    let mapping_repo = Arc::new(FakeCsvMappingRepo::new());

    let uc = UploadImport::new(
        decoder,
        batch_repo.clone(),
        movement_repo.clone(),
        mapping_repo.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: "test.csv".into(),
            bytes: csv,
            currency: "EUR".into(),
            mapping: Some(ColumnMapping {
                date_column: "Date".into(),
                amount_column: "Amount".into(),
                description_column: "Description".into(),
                origin_account_column: None,
            }),
        })
        .await
        .unwrap();

    assert!(result.origin_account_values.is_empty());
    assert!(result.valid[0].origin_account.is_none());
}
