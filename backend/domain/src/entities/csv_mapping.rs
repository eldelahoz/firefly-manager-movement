use chrono::NaiveDateTime;

use crate::ids::CsvMappingId;
use crate::value_objects::HeaderSignature;

/// CSV mapping memorized by header signature (RF-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvMapping {
    pub id: CsvMappingId,
    pub header_signature: HeaderSignature,
    pub mapping_json: String,
    pub created_at: NaiveDateTime,
}
