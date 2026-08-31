//! `MysqlMatchRepository` - implements `MatchRepository`.
//! The module is named `match_repo` because `match` is a reserved keyword.

use std::collections::HashSet;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::NewMatch;
use ffm_application::error::ApplicationError;
use ffm_application::ports::MatchRepository;
use ffm_domain::entities::Match;
use ffm_domain::ids::ReconciliationId;

use crate::models::{MatchModel, NewMatchModel};
use crate::persistence::pool::DbPool;
use crate::schema::{ff3_transactions, matches};

pub struct MysqlMatchRepository {
    pool: DbPool,
}

impl MysqlMatchRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MatchRepository for MysqlMatchRepository {
    async fn insert_batch(&self, matches: Vec<NewMatch>) -> Result<(), ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        let models: Vec<NewMatchModel> = matches.into_iter().map(Into::into).collect();
        if models.is_empty() {
            return Ok(());
        }
        diesel::insert_into(crate::schema::matches::table)
            .values(&models)
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        Ok(())
    }

    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Match>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        matches::table
            .filter(matches::reconciliation_id.eq(reconciliation_id.as_i64()))
            .order(matches::id.asc())
            .load::<MatchModel>(&mut conn)
            .await
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }

    async fn matched_ff3_ids(&self, ff3_ids: &[i64]) -> Result<HashSet<i64>, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;
        if ff3_ids.is_empty() {
            return Ok(HashSet::new());
        }
        ff3_transactions::table
            .inner_join(matches::table.on(matches::ff3_transaction_id.eq(ff3_transactions::id)))
            .filter(ff3_transactions::ff3_id.eq_any(ff3_ids))
            .select(ff3_transactions::ff3_id)
            .distinct()
            .load::<i64>(&mut conn)
            .await
            .map(|v| v.into_iter().collect())
            .map_err(|e| ApplicationError::Database(e.to_string()))
    }
}
