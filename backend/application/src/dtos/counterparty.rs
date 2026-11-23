//! Counterparty suggestions (RF-08 v0.11): what the app learned and what
//! Firefly III history shows, scoped by batch account and direction.

use ffm_domain::transaction_kind::Direction;

/// A counterparty choice: an existing account (`ff3_id`, with its name),
/// a name to create, or Cash.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CounterpartyRef {
    pub ff3_id: Option<i64>,
    pub name: Option<String>,
    pub cash: bool,
}

/// One learned entry, unique per `(account_ff3_id, direction, desc_key)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterpartyMemoryEntry {
    pub account_ff3_id: i64,
    pub direction: Direction,
    /// `description_key` of the statement description.
    pub desc_key: String,
    pub counterparty: CounterpartyRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionSource {
    Learned,
    History,
}

impl SuggestionSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            SuggestionSource::Learned => "learned",
            SuggestionSource::History => "history",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedCounterparty {
    pub counterparty: CounterpartyRef,
    pub source: SuggestionSource,
}
