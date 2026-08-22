use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use ffm_application::dtos::NewCsvMapping;
use ffm_domain::entities::CsvMapping;
use ffm_domain::ids::CsvMappingId;
use ffm_domain::value_objects::HeaderSignature;

use crate::schema::csv_mappings;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = csv_mappings)]
pub struct CsvMappingModel {
    pub id: i64,
    pub header_signature: String,
    pub mapping_json: JsonValue,
    pub created_at: NaiveDateTime,
}

impl From<CsvMappingModel> for CsvMapping {
    fn from(m: CsvMappingModel) -> Self {
        Self {
            id: CsvMappingId::new(m.id),
            header_signature: HeaderSignature::from_str(&m.header_signature).unwrap(),
            mapping_json: m.mapping_json.to_string(),
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = csv_mappings)]
pub struct NewCsvMappingModel {
    pub header_signature: String,
    pub mapping_json: JsonValue,
}

impl From<NewCsvMapping> for NewCsvMappingModel {
    fn from(d: NewCsvMapping) -> Self {
        Self {
            header_signature: d.header_signature.as_str().to_string(),
            mapping_json: serde_json::from_str(&d.mapping_json).unwrap_or(JsonValue::Null),
        }
    }
}
