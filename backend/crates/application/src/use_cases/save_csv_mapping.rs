//! SaveCsvMapping use case.
//! Stores a column mapping keyed by the normalized header signature (RF-02).

use crate::csv::ColumnMapping;
use crate::dtos::NewCsvMapping;
use crate::error::ApplicationError;
use crate::ports::CsvMappingRepository;

use ffm_domain::ids::CsvMappingId;
use ffm_domain::value_objects::HeaderSignature;

pub struct SaveCsvMapping<R> {
    repo: R,
}

impl<R> SaveCsvMapping<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: CsvMappingRepository> SaveCsvMapping<R> {
    pub async fn execute(
        &self,
        headers: Vec<String>,
        mapping: ColumnMapping,
    ) -> Result<CsvMappingId, ApplicationError> {
        let signature = HeaderSignature::from_headers(&headers);
        let mapping_json = serde_json::to_string(&mapping)
            .map_err(|e| ApplicationError::Parse(format!("failed to serialize mapping: {e}")))?;
        self.repo
            .upsert(NewCsvMapping {
                header_signature: signature,
                mapping_json,
            })
            .await
    }
}
