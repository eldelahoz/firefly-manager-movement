//! `MysqlMovementRepository` — implements `MovementRepository`.

use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewMovement;
use ffm_application::error::ApplicationError;
use ffm_application::ports::MovementRepository;
use ffm_domain::entities::Movement;
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::MovementStatus;

use crate::models::{MovementModel, NewMovementModel};
use crate::persistence::pool::DbPool;
use crate::schema::movements;

pub struct MysqlMovementRepository {
    pool: DbPool,
}

impl MysqlMovementRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MovementRepository for MysqlMovementRepository {
    async fn insert_batch(&self, movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let models: Vec<NewMovementModel> = movements.into_iter().map(Into::into).collect();
        if models.is_empty() {
            return Ok(());
        }
        diesel::insert_into(movements::table)
            .values(&models)
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        Ok(())
    }

    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        movements::table
            .filter(movements::batch_id.eq(batch_id.as_i64()))
            .order(movements::line_no.asc())
            .load::<MovementModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }

    async fn count_by_batch(&self) -> Result<HashMap<BatchId, i64>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let ids = movements::table
            .select(movements::batch_id)
            .load::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let mut counts: HashMap<BatchId, i64> = HashMap::new();
        for id in ids {
            *counts.entry(BatchId::new(id)).or_insert(0) += 1;
        }
        Ok(counts)
    }

    async fn update_status(
        &self,
        id: MovementId,
        status: MovementStatus,
    ) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        diesel::update(movements::table.filter(movements::id.eq(id.as_i64())))
            .set(movements::status.eq(status.as_str()))
            .execute(&mut conn)
            .await
            .map(|_| ())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }

    async fn delete_by_batch(&self, batch_id: BatchId) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        diesel::delete(movements::table.filter(movements::batch_id.eq(batch_id.as_i64())))
            .execute(&mut conn)
            .await
            .map(|_| ())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }
}
