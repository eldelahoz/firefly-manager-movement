//! `MysqlReconciliationRepository` - implements `ReconciliationRepository`.

use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use ffm_application::dtos::{NewReconciliation, ReconciliationRun};
use ffm_application::error::ApplicationError;
use ffm_application::ports::ReconciliationRepository;
use ffm_domain::entities::Reconciliation;
use ffm_domain::ids::{BatchId, ReconciliationId};

use crate::models::{
    NewFf3TransactionModel, NewMatchModel, NewReconciliationModel, ReconciliationModel,
};
use crate::persistence::pool::DbPool;
use crate::schema::{ff3_transactions, matches, movements, reconciliations};

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
        use diesel_async::TransactionManager;

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ApplicationError::Database(e.to_string()))?;

        // Single transaction (R8): all statements commit together or none.
        // (Reborrow the inner connection of the pool object.)
        let inner: &mut diesel_async::AsyncMysqlConnection = &mut conn;
        <diesel_async::AnsiTransactionManager as TransactionManager<
            diesel_async::AsyncMysqlConnection,
        >>::begin_transaction(inner)
        .await
        .map_err(map_diesel_err)?;

        match save_run_tx(&mut conn, run).await {
            Ok(recon_id) => {
                let inner: &mut diesel_async::AsyncMysqlConnection = &mut conn;
                <diesel_async::AnsiTransactionManager as TransactionManager<
                    diesel_async::AsyncMysqlConnection,
                >>::commit_transaction(inner)
                .await
                .map_err(map_diesel_err)?;
                Ok(ReconciliationId::new(recon_id))
            }
            Err(e) => {
                // Best-effort rollback; the original error is the meaningful one.
                let inner: &mut diesel_async::AsyncMysqlConnection = &mut conn;
                let _ = <diesel_async::AnsiTransactionManager as TransactionManager<
                    diesel_async::AsyncMysqlConnection,
                >>::rollback_transaction(inner)
                .await;
                Err(e)
            }
        }
    }
}

// Body of the atomic run persistence, executed inside a transaction.
async fn save_run_tx(
    conn: &mut diesel_async::AsyncMysqlConnection,
    run: ReconciliationRun,
) -> Result<i64, ApplicationError> {
    // 1. Reconciliation snapshot.
    diesel::insert_into(reconciliations::table)
        .values(&NewReconciliationModel::from(run.reconciliation))
        .execute(conn)
        .await
        .map_err(map_diesel_err)?;
    let recon_id: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "LAST_INSERT_ID()",
    ))
    .get_result(conn)
    .await
    .map_err(map_diesel_err)?;

    // 2. Per-run FF3 transaction cache (row-by-row to capture each
    //    generated local id).
    let mut ff3_local: HashMap<i64, i64> = HashMap::new();
    for t in run.ff3_transactions {
        diesel::insert_into(ff3_transactions::table)
            .values(&NewFf3TransactionModel {
                reconciliation_id: recon_id,
                ff3_id: t.ff3_id,
                date: t.date,
                amount_cents: t.amount_cents.as_i64(),
                description: t.description,
                is_duplicate: t.is_duplicate,
            })
            .execute(conn)
            .await
            .map_err(map_diesel_err)?;
        let local: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "LAST_INSERT_ID()",
        ))
        .get_result(conn)
        .await
        .map_err(map_diesel_err)?;
        ff3_local.insert(t.ff3_id, local);
    }

    // 3. 1-to-1 matches (resolve ff3_id → local row id).
    for m in run.matches {
        let ff3_transaction_id = ff3_local.get(&m.ff3_id).copied().ok_or_else(|| {
            ApplicationError::Database(format!("match references unknown ff3_id {}", m.ff3_id))
        })?;
        diesel::insert_into(matches::table)
            .values(&NewMatchModel {
                reconciliation_id: recon_id,
                movement_id: m.movement_id.as_i64(),
                ff3_transaction_id,
            })
            .execute(conn)
            .await
            .map_err(map_diesel_err)?;
    }

    // 4. Movement statuses.
    for (movement_id, status) in run.movement_status_updates {
        diesel::update(movements::table.filter(movements::id.eq(movement_id.as_i64())))
            .set(movements::status.eq(status.as_str()))
            .execute(conn)
            .await
            .map_err(map_diesel_err)?;
    }

    Ok(recon_id)
}
