//! `MysqlFf3TransactionRepository` — implements `Ff3TransactionRepository`.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewFf3Transaction;
use ffm_application::error::ApplicationError;
use ffm_application::ports::Ff3TransactionRepository;
use ffm_domain::entities::Ff3Transaction;
use ffm_domain::ids::ReconciliationId;

use crate::models::{Ff3TransactionModel, NewFf3TransactionModel};
use crate::persistence::pool::DbPool;
use crate::schema::ff3_transactions;

pub struct MysqlFf3TransactionRepository {
    pool: DbPool,
}

impl MysqlFf3TransactionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Ff3TransactionRepository for MysqlFf3TransactionRepository {
    async fn insert_batch(
        &self,
        transactions: Vec<NewFf3Transaction>,
    ) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let models: Vec<NewFf3TransactionModel> =
            transactions.into_iter().map(Into::into).collect();
        if models.is_empty() {
            return Ok(());
        }
        diesel::insert_into(ff3_transactions::table)
            .values(&models)
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        Ok(())
    }

    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        ff3_transactions::table
            .filter(ff3_transactions::reconciliation_id.eq(reconciliation_id.as_i64()))
            .order(ff3_transactions::date.asc())
            .load::<Ff3TransactionModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }

    async fn latest_by_ff3_id(
        &self,
        ff3_id: i64,
    ) -> Result<Option<Ff3Transaction>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        match ff3_transactions::table
            .filter(ff3_transactions::ff3_id.eq(ff3_id))
            .order(ff3_transactions::id.desc())
            .first::<Ff3TransactionModel>(&mut conn)
            .await
        {
            Ok(t) => Ok(Some(t.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }
}
