//! `MysqlCsvMappingRepository` — implements `CsvMappingRepository`.
//! Upsert by `header_signature` (UNIQUE) with `ON DUPLICATE KEY UPDATE`;
//! `LAST_INSERT_ID(id)` in the UPDATE keeps the row id accessible to a
//! subsequent `LAST_INSERT_ID()` (whether it inserted or updated).

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

pub struct MysqlCsvMappingRepository {
    pool: DbPool,
}

impl MysqlCsvMappingRepository {
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
impl CsvMappingRepository for MysqlCsvMappingRepository {
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
        diesel::sql_query(
            "INSERT INTO csv_mappings (header_signature, mapping_json) VALUES (?, ?) \
             ON DUPLICATE KEY UPDATE mapping_json = VALUES(mapping_json), \
             id = LAST_INSERT_ID(id)",
        )
        .bind::<diesel::sql_types::Varchar, _>(&model.header_signature)
        .bind::<diesel::sql_types::Json, _>(&model.mapping_json)
        .execute(&mut conn)
        .await
        .map_err(map_diesel_err)?;
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "LAST_INSERT_ID()",
        ))
        .get_result::<i64>(&mut conn)
        .await
        .map(CsvMappingId::new)
        .map_err(map_diesel_err)
    }
}
