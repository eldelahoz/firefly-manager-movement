//! `SqliteReconciliationRepository` - implements `ReconciliationRepository`.

use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};

use ffm_application::dtos::{NewReconciliation, ReconciliationRun};
use ffm_application::error::ApplicationError;
use ffm_application::ports::ReconciliationRepository;
use ffm_domain::entities::Reconciliation;
use ffm_domain::ids::{BatchId, ReconciliationId};

use crate::models::{
    NewFf3TransactionModel, NewMatchModel, NewReconciliationModel, ReconciliationModel,
};
use crate::persistence::pool::{DbConnection, DbPool};
use crate::schema::{ff3_transactions, matches, movements, reconciliations};

pub struct SqliteReconciliationRepository {
    pool: DbPool,
}

impl SqliteReconciliationRepository {
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
impl ReconciliationRepository for SqliteReconciliationRepository {
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
            .returning(reconciliations::id)
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
            // `id` is monotonic: deterministic "latest" even when two runs
            // share the same `ran_at` second.
            .order(reconciliations::id.desc())
            .first::<ReconciliationModel>(&mut conn)
            .await
        {
            Ok(r) => Ok(Some(r.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(ApplicationError::Database(e.to_string())),
        }
    }

    async fn save_run(&self, run: ReconciliationRun) -> Result<ReconciliationId, ApplicationError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;

        // Single transaction (R8): all statements commit together or none.
        conn.transaction(async |c| save_run_tx(c, run).await)
            .await
            .map(ReconciliationId::new)
            .map_err(map_diesel_err)
    }
}

// Body of the atomic run persistence, executed inside a transaction.
async fn save_run_tx(conn: &mut DbConnection, run: ReconciliationRun) -> QueryResult<i64> {
    // 1. Reconciliation snapshot.
    let recon_id: i64 = diesel::insert_into(reconciliations::table)
        .values(&NewReconciliationModel::from(run.reconciliation))
        .returning(reconciliations::id)
        .get_result(conn)
        .await?;

    // 2. Per-run FF3 transaction cache (row-by-row to capture each
    //    generated local id).
    let mut ff3_local: HashMap<(i64, i64), i64> = HashMap::new();
    for t in run.ff3_transactions {
        let local: i64 = diesel::insert_into(ff3_transactions::table)
            .values(&NewFf3TransactionModel {
                reconciliation_id: recon_id,
                ff3_id: t.ff3_id,
                ff3_journal_id: t.ff3_journal_id,
                date: t.date,
                amount_cents: t.amount_cents.as_i64(),
                description: t.description,
                is_duplicate: t.is_duplicate,
                counterparty_name: t.counterparty_name,
                counterparty_type: t.counterparty_type,
            })
            .returning(ff3_transactions::id)
            .get_result(conn)
            .await?;
        ff3_local.insert((t.ff3_id, t.ff3_journal_id), local);
    }

    // 3. 1-to-1 matches (resolve (ff3_id, ff3_journal_id) → local row id).
    for m in run.matches {
        let ff3_transaction_id = ff3_local
            .get(&(m.ff3_id, m.ff3_journal_id))
            .copied()
            .ok_or_else(|| {
                diesel::result::Error::QueryBuilderError(
                    format!("match references unknown ff3_id {}", m.ff3_id).into(),
                )
            })?;
        diesel::insert_into(matches::table)
            .values(&NewMatchModel {
                reconciliation_id: recon_id,
                movement_id: m.movement_id.as_i64(),
                ff3_transaction_id,
            })
            .execute(conn)
            .await?;
    }

    // 4. Movement statuses.
    for (movement_id, status) in run.movement_status_updates {
        diesel::update(movements::table.filter(movements::id.eq(movement_id.as_i64())))
            .set(movements::status.eq(status.as_str()))
            .execute(conn)
            .await?;
    }

    Ok(recon_id)
}
