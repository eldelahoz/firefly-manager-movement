use std::collections::HashMap;

use async_trait::async_trait;

use ffm_domain::ids::{BatchId, MovementId};

use crate::dtos::{CounterpartyMemoryEntry, CounterpartyRef};
use crate::error::ApplicationError;

/// Learned counterparties and per-movement history suggestions (RF-08 v0.11).
#[async_trait]
pub trait CounterpartySuggestionRepository: Send + Sync {
    /// Upsert on `(account_ff3_id, direction, desc_key)`: the latest choice
    /// replaces the previous one and `uses` grows by one.
    async fn remember(&self, entry: CounterpartyMemoryEntry) -> Result<(), ApplicationError>;
    async fn learned(
        &self,
        account_ff3_id: i64,
    ) -> Result<Vec<CounterpartyMemoryEntry>, ApplicationError>;
    /// Clears the history suggestion of every movement of the batch, then
    /// stores the given ones.
    async fn save_history(
        &self,
        batch_id: BatchId,
        suggestions: Vec<(MovementId, CounterpartyRef)>,
    ) -> Result<(), ApplicationError>;
    async fn history_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<HashMap<MovementId, CounterpartyRef>, ApplicationError>;
}

/// Test-only in-memory store with the same upsert semantics.
#[doc(hidden)]
#[derive(Default)]
pub struct InMemoryCounterpartySuggestions {
    /// Learned entries with their `uses`.
    pub learned: std::sync::Mutex<Vec<(CounterpartyMemoryEntry, i64)>>,
    pub history: std::sync::Mutex<HashMap<BatchId, HashMap<MovementId, CounterpartyRef>>>,
}

#[async_trait]
impl CounterpartySuggestionRepository for InMemoryCounterpartySuggestions {
    async fn remember(&self, entry: CounterpartyMemoryEntry) -> Result<(), ApplicationError> {
        let mut learned = self.learned.lock().unwrap();
        match learned.iter_mut().find(|(e, _)| {
            (e.account_ff3_id, e.direction, &e.desc_key)
                == (entry.account_ff3_id, entry.direction, &entry.desc_key)
        }) {
            Some((e, uses)) => {
                e.counterparty = entry.counterparty;
                *uses += 1;
            }
            None => learned.push((entry, 1)),
        }
        Ok(())
    }
    async fn learned(
        &self,
        account_ff3_id: i64,
    ) -> Result<Vec<CounterpartyMemoryEntry>, ApplicationError> {
        Ok(self
            .learned
            .lock()
            .unwrap()
            .iter()
            .filter(|(e, _)| e.account_ff3_id == account_ff3_id)
            .map(|(e, _)| e.clone())
            .collect())
    }
    async fn save_history(
        &self,
        batch_id: BatchId,
        suggestions: Vec<(MovementId, CounterpartyRef)>,
    ) -> Result<(), ApplicationError> {
        self.history
            .lock()
            .unwrap()
            .insert(batch_id, suggestions.into_iter().collect());
        Ok(())
    }
    async fn history_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<HashMap<MovementId, CounterpartyRef>, ApplicationError> {
        Ok(self
            .history
            .lock()
            .unwrap()
            .get(&batch_id)
            .cloned()
            .unwrap_or_default())
    }
}
