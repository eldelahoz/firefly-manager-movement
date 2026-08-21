//! `MysqlCreationRepository` — implements `CreationRepository`.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewCreation;
use ffm_application::error::ApplicationError;
use ffm_application::ports::CreationRepository;
use ffm_domain::entities::Creation;
use ffm_domain::ids::{BatchId, CreationId, MovementId};

use crate::models::{CreationModel, NewCreationModel};
use crate::persistence::pool::DbPool;
use crate::schema::{creations, movements};

pub struct MysqlCreationRepository {
    pool: DbPool,
}

impl MysqlCreationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn map_diesel_err(e: diesel::result::Error) -> ApplicationError {
    use diesel::result::Error as E;
    match e {
        E::NotFound => ApplicationError::NotFound("creation".into()),
        other => ApplicationError::Database(other.to_string()),
    }
}

#[async_trait]
impl CreationRepository for MysqlCreationRepository {
    async fn insert(&self, creation: NewCreation) -> Result<CreationId, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let model = NewCreationModel::from(creation);
        diesel::insert_into(creations::table)
            .values(&model)
            .execute(&mut conn)
            .await
            .map_err(map_diesel_err)?;
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "LAST_INSERT_ID()",
        ))
        .get_result::<i64>(&mut conn)
        .await
        .map(CreationId::new)
        .map_err(map_diesel_err)
    }

    async fn by_movement(
        &self,
        movement_id: MovementId,
    ) -> Result<Option<Creation>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        match creations::table
            .filter(creations::movement_id.eq(movement_id.as_i64()))
            .first::<CreationModel>(&mut conn)
            .await
        {
            Ok(c) => Ok(Some(c.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }

    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Creation>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        // `creations` has no batch_id: the batch is reached via movements.
        creations::table
            .inner_join(movements::table.on(movements::id.eq(creations::movement_id)))
            .filter(movements::batch_id.eq(batch_id.as_i64()))
            .select(creations::all_columns)
            .load::<CreationModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }
}
