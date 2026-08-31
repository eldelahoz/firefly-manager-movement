//! `MysqlBatchRepository` - implements `BatchRepository`.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewBatch;
use ffm_application::error::ApplicationError;
use ffm_application::ports::BatchRepository;
use ffm_domain::entities::Batch;
use ffm_domain::ids::{BatchId, Ff3AccountId};
use ffm_domain::value_objects::BatchStatus;

use crate::models::{BatchModel, NewBatchModel};
use crate::persistence::pool::DbPool;
use crate::schema::batches;

pub struct MysqlBatchRepository {
    pool: DbPool,
}

impl MysqlBatchRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn map_diesel_err(e: diesel::result::Error) -> ApplicationError {
    use diesel::result::Error as E;
    match e {
        E::NotFound => ApplicationError::NotFound("batch".into()),
        other => ApplicationError::Database(other.to_string()),
    }
}

#[async_trait]
impl BatchRepository for MysqlBatchRepository {
    async fn insert(&self, batch: NewBatch) -> Result<BatchId, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let model = NewBatchModel::from(batch);
        diesel::insert_into(batches::table)
            .values(&model)
            .execute(&mut conn)
            .await
            .map_err(map_diesel_err)?;
        // MySQL does not support RETURNING in diesel 2.x: the auto-increment
        // is read with `LAST_INSERT_ID()` on the same connection.
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "LAST_INSERT_ID()",
        ))
        .get_result::<i64>(&mut conn)
        .await
        .map(BatchId::new)
        .map_err(map_diesel_err)
    }

    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        batches::table
            .find(id.as_i64())
            .first::<BatchModel>(&mut conn)
            .await
            .map(Into::into)
            .map_err(map_diesel_err)
    }

    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        batches::table
            .order(batches::id.desc())
            .load::<BatchModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(map_diesel_err)
    }

    async fn update_status(
        &self,
        id: BatchId,
        status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        diesel::update(batches::table.filter(batches::id.eq(id.as_i64())))
            .set(batches::status.eq(status.as_str()))
            .execute(&mut conn)
            .await
            .map(|_| ())
            .map_err(map_diesel_err)
    }

    async fn update_account(
        &self,
        id: BatchId,
        account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        diesel::update(batches::table.filter(batches::id.eq(id.as_i64())))
            .set(batches::account_id.eq(Some(account_id.as_i64())))
            .execute(&mut conn)
            .await
            .map(|_| ())
            .map_err(map_diesel_err)
    }

    async fn find_by_filename(&self, filename: &str) -> Result<Vec<Batch>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        // MySQL collation (utf8mb4_*) es case-insensitive: `=` basta.
        batches::table
            .filter(batches::filename.eq(filename))
            .order(batches::id.desc())
            .load::<BatchModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(map_diesel_err)
    }

    async fn delete(&self, id: BatchId) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        diesel::delete(batches::table.filter(batches::id.eq(id.as_i64())))
            .execute(&mut conn)
            .await
            .map(|_| ())
            .map_err(map_diesel_err)
    }
}
