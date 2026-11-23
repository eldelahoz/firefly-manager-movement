use diesel::prelude::*;

use ffm_application::dtos::{CounterpartyMemoryEntry, CounterpartyRef};
use ffm_application::error::ApplicationError;
use ffm_domain::transaction_kind::Direction;

use crate::schema::counterparty_memory;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = counterparty_memory)]
pub struct CounterpartyMemoryModel {
    pub account_ff3_id: i64,
    pub direction: String,
    pub desc_key: String,
    pub counterparty_ff3_id: Option<i64>,
    pub counterparty_name: Option<String>,
    pub is_cash: bool,
}

impl TryFrom<CounterpartyMemoryModel> for CounterpartyMemoryEntry {
    type Error = ApplicationError;
    fn try_from(m: CounterpartyMemoryModel) -> Result<Self, Self::Error> {
        let direction = match m.direction.as_str() {
            "out" => Direction::Out,
            "in" => Direction::In,
            other => {
                return Err(ApplicationError::Database(format!(
                    "invalid counterparty_memory direction: {other}"
                )))
            }
        };
        Ok(Self {
            account_ff3_id: m.account_ff3_id,
            direction,
            desc_key: m.desc_key,
            counterparty: CounterpartyRef {
                ff3_id: m.counterparty_ff3_id,
                name: m.counterparty_name,
                cash: m.is_cash,
            },
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = counterparty_memory)]
pub struct NewCounterpartyMemoryModel {
    pub account_ff3_id: i64,
    pub direction: &'static str,
    pub desc_key: String,
    pub counterparty_ff3_id: Option<i64>,
    pub counterparty_name: Option<String>,
    pub is_cash: bool,
}

impl From<CounterpartyMemoryEntry> for NewCounterpartyMemoryModel {
    fn from(e: CounterpartyMemoryEntry) -> Self {
        Self {
            account_ff3_id: e.account_ff3_id,
            direction: e.direction.as_str(),
            desc_key: e.desc_key,
            counterparty_ff3_id: e.counterparty.ff3_id,
            counterparty_name: e.counterparty.name,
            is_cash: e.counterparty.cash,
        }
    }
}
