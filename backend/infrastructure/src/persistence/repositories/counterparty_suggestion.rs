//! `SqliteCounterpartySuggestionRepository` - implements `CounterpartySuggestionRepository`
//! (RF-08 v0.11): `counterparty_memory` plus the `movements.history_*` columns.

use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_async::{AsyncConnection, RunQueryDsl};

use ffm_application::dtos::{CounterpartyMemoryEntry, CounterpartyRef};
use ffm_application::error::ApplicationError;
use ffm_application::ports::CounterpartySuggestionRepository;
use ffm_domain::ids::{BatchId, MovementId};

use crate::models::{CounterpartyMemoryModel, NewCounterpartyMemoryModel};
use crate::persistence::pool::DbPool;
use crate::schema::{counterparty_memory as cm, movements};

pub struct SqliteCounterpartySuggestionRepository {
    pool: DbPool,
}

impl SqliteCounterpartySuggestionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn db_err(e: impl ToString) -> ApplicationError {
    ApplicationError::Database(e.to_string())
}

#[async_trait]
impl CounterpartySuggestionRepository for SqliteCounterpartySuggestionRepository {
    async fn remember(&self, entry: CounterpartyMemoryEntry) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().await.map_err(db_err)?;
        diesel::insert_into(cm::table)
            .values(NewCounterpartyMemoryModel::from(entry))
            .on_conflict((cm::account_ff3_id, cm::direction, cm::desc_key))
            .do_update()
            .set((
                cm::counterparty_ff3_id.eq(excluded(cm::counterparty_ff3_id)),
                cm::counterparty_name.eq(excluded(cm::counterparty_name)),
                cm::is_cash.eq(excluded(cm::is_cash)),
                cm::uses.eq(cm::uses + 1),
                cm::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map(|_| ())
            .map_err(db_err)
    }

    async fn learned(
        &self,
        account_ff3_id: i64,
    ) -> Result<Vec<CounterpartyMemoryEntry>, ApplicationError> {
        let mut conn = self.pool.get().await.map_err(db_err)?;
        cm::table
            .filter(cm::account_ff3_id.eq(account_ff3_id))
            .select(CounterpartyMemoryModel::as_select())
            .load::<CounterpartyMemoryModel>(&mut conn)
            .await
            .map_err(db_err)?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    async fn save_history(
        &self,
        batch_id: BatchId,
        suggestions: Vec<(MovementId, CounterpartyRef)>,
    ) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().await.map_err(db_err)?;
        conn.transaction(async |c| {
            diesel::update(movements::table.filter(movements::batch_id.eq(batch_id.as_i64())))
                .set((
                    movements::history_ff3_id.eq(None::<i64>),
                    movements::history_name.eq(None::<String>),
                    movements::history_cash.eq(None::<bool>),
                ))
                .execute(c)
                .await?;
            for (movement_id, cp) in suggestions {
                diesel::update(movements::table.filter(movements::id.eq(movement_id.as_i64())))
                    .set((
                        movements::history_ff3_id.eq(cp.ff3_id),
                        movements::history_name.eq(cp.name),
                        movements::history_cash.eq(Some(cp.cash)),
                    ))
                    .execute(c)
                    .await?;
            }
            Ok::<_, diesel::result::Error>(())
        })
        .await
        .map_err(db_err)
    }

    async fn history_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<HashMap<MovementId, CounterpartyRef>, ApplicationError> {
        let mut conn = self.pool.get().await.map_err(db_err)?;
        // `history_cash` is set on every stored suggestion: NULL = none.
        let rows: Vec<(i64, Option<i64>, Option<String>, Option<bool>)> = movements::table
            .filter(movements::batch_id.eq(batch_id.as_i64()))
            .filter(movements::history_cash.is_not_null())
            .select((
                movements::id,
                movements::history_ff3_id,
                movements::history_name,
                movements::history_cash,
            ))
            .load(&mut conn)
            .await
            .map_err(db_err)?;
        Ok(rows
            .into_iter()
            .map(|(id, ff3_id, name, cash)| {
                let cash = cash.unwrap_or(false);
                (MovementId::new(id), CounterpartyRef { ff3_id, name, cash })
            })
            .collect())
    }
}
