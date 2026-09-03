use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};

use ffm_application::csv::ColumnMapping;
use ffm_application::dtos::NewCsvMapping;
use ffm_application::error::ApplicationError;
use ffm_application::ports::CsvMappingRepository;
use ffm_application::use_cases::save_csv_mapping::SaveCsvMapping;

use ffm_domain::entities::CsvMapping;
use ffm_domain::ids::CsvMappingId;
use ffm_domain::value_objects::HeaderSignature;

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

struct FakeCsvMappingRepo {
    stored: Mutex<Vec<CsvMapping>>,
}

impl FakeCsvMappingRepo {
    fn new() -> Self {
        Self {
            stored: Mutex::new(Vec::new()),
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
            .stored
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.header_signature.as_str() == signature.as_str())
            .cloned())
    }

    async fn upsert(&self, mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError> {
        let mut store = self.stored.lock().unwrap();
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

#[tokio::test]
async fn saves_mapping_by_header_signature() {
    let repo = Arc::new(FakeCsvMappingRepo::new());
    let uc = SaveCsvMapping::new(repo.clone());

    let headers = vec![
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
    ];
    let mapping = ColumnMapping {
        date_column: "Date".into(),
        amount_column: "Amount".into(),
        description_column: "Description".into(),
        origin_account_column: None,
    };

    let id = uc.execute(headers.clone(), mapping.clone()).await.unwrap();
    assert_eq!(id.as_i64(), 1);

    let sig = HeaderSignature::from_headers(&headers);
    let stored = repo.by_signature(sig).await.unwrap().unwrap();
    let roundtrip: ColumnMapping = serde_json::from_str(&stored.mapping_json).unwrap();
    assert_eq!(roundtrip.date_column, "Date");
    assert_eq!(roundtrip.amount_column, "Amount");
    assert_eq!(roundtrip.description_column, "Description");
}

#[tokio::test]
async fn upsert_overwrites_existing_mapping() {
    let repo = Arc::new(FakeCsvMappingRepo::new());
    let uc = SaveCsvMapping::new(repo.clone());

    let headers = vec![
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
    ];

    let id1 = uc
        .execute(
            headers.clone(),
            ColumnMapping {
                date_column: "Date".into(),
                amount_column: "Amount".into(),
                description_column: "Description".into(),
                origin_account_column: None,
            },
        )
        .await
        .unwrap();

    let id2 = uc
        .execute(
            headers.clone(),
            ColumnMapping {
                date_column: "Date".into(),
                amount_column: "Amount".into(),
                description_column: "Desc".into(),
                origin_account_column: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(id1.as_i64(), id2.as_i64());

    let sig = HeaderSignature::from_headers(&headers);
    let stored = repo.by_signature(sig).await.unwrap().unwrap();
    let roundtrip: ColumnMapping = serde_json::from_str(&stored.mapping_json).unwrap();
    assert_eq!(roundtrip.description_column, "Desc");
}
