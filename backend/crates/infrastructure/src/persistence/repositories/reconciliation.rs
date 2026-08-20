//! `MysqlReconciliationRepository` — implementa `ReconciliationRepository`.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewReconciliation;
use ffm_application::error::ApplicationError;
use ffm_application::ports::ReconciliationRepository;
use ffm_domain::entities::Reconciliation;
use ffm_domain::ids::{BatchId, ReconciliationId};

use crate::models::{NewReconciliationModel, ReconciliationModel};
use crate::persistence::pool::DbPool;
use crate::schema::reconciliations;

pub struct MysqlReconciliationRepository {
    pool: DbPool,
}

impl MysqlReconciliationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn map_diesel_err(e: diesel::result::Error) -> ApplicationError {
    use diesel::result::Error as E;
    match e {
        E::NotFound => ApplicationError::NotFound("reconciliation".into()),
        other => ApplicationError::Database(other.to_string()),
    }
}

#[async_trait]
impl ReconciliationRepository for MysqlReconciliationRepository {
    async fn insert(
        &self,
        reconciliation: NewReconciliation,
    ) -> Result<ReconciliationId, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let model = NewReconciliationModel::from(reconciliation);
        diesel::insert_into(reconciliations::table)
            .values(&model)
            .execute(&mut conn)
            .await
            .map_err(map_diesel_err)?;
        diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "LAST_INSERT_ID()",
        ))
        .get_result::<i64>(&mut conn)
        .await
        .map(ReconciliationId::new)
        .map_err(map_diesel_err)
    }

    async fn latest_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<Option<Reconciliation>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        match reconciliations::table
            .filter(reconciliations::batch_id.eq(batch_id.as_i64()))
            .order(reconciliations::ran_at.desc())
            .first::<ReconciliationModel>(&mut conn)
            .await
        {
            Ok(r) => Ok(Some(r.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }
}
