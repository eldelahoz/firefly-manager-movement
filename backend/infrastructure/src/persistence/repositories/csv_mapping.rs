//! `SqliteCsvMappingRepository` - implements `CsvMappingRepository`.
//! Upsert by `header_signature` (UNIQUE) with `ON CONFLICT DO UPDATE ... RETURNING id`
//! (returns the row id whether it inserted or updated).

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewCsvMapping;
use ffm_application::error::ApplicationError;
use ffm_application::ports::CsvMappingRepository;
use ffm_domain::entities::CsvMapping;
use ffm_domain::ids::CsvMappingId;
use ffm_domain::value_objects::HeaderSignature;

use crate::models::{CsvMappingModel, NewCsvMappingModel};
use crate::persistence::pool::DbPool;
use crate::schema::csv_mappings;

pub struct SqliteCsvMappingRepository {
    pool: DbPool,
}

impl SqliteCsvMappingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn map_diesel_err(e: diesel::result::Error) -> ApplicationError {
    use diesel::result::Error as E;
    match e {
        E::NotFound => ApplicationError::NotFound("csv_mapping".into()),
        other => ApplicationError::Database(other.to_string()),
    }
}

#[async_trait]
impl CsvMappingRepository for SqliteCsvMappingRepository {
    async fn by_signature(
        &self,
        signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        match csv_mappings::table
            .filter(csv_mappings::header_signature.eq(signature.as_str()))
            .first::<CsvMappingModel>(&mut conn)
            .await
        {
            Ok(m) => Ok(Some(m.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }

    async fn upsert(&self, mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let model = NewCsvMappingModel::from(mapping);
        diesel::insert_into(csv_mappings::table)
            .values(&model)
            .on_conflict(csv_mappings::header_signature)
            .do_update()
            .set(
                csv_mappings::mapping_json.eq(diesel::upsert::excluded(csv_mappings::mapping_json)),
            )
            .returning(csv_mappings::id)
            .get_result::<i64>(&mut conn)
            .await
            .map(CsvMappingId::new)
            .map_err(map_diesel_err)
    }
}
