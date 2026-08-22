use chrono::NaiveDateTime;
use diesel::prelude::*;

use ffm_application::dtos::NewMatch;
use ffm_domain::entities::Match;
use ffm_domain::ids::{Ff3TransactionId, MatchId, MovementId, ReconciliationId};

use crate::schema::matches;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = matches)]
pub struct MatchModel {
    pub id: i64,
    pub reconciliation_id: i64,
    pub movement_id: i64,
    pub ff3_transaction_id: i64,
    pub matched_at: NaiveDateTime,
}

impl From<MatchModel> for Match {
    fn from(m: MatchModel) -> Self {
        Self {
            id: MatchId::new(m.id),
            reconciliation_id: ReconciliationId::new(m.reconciliation_id),
            movement_id: MovementId::new(m.movement_id),
            ff3_transaction_id: Ff3TransactionId::new(m.ff3_transaction_id),
            matched_at: m.matched_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = matches)]
pub struct NewMatchModel {
    pub reconciliation_id: i64,
    pub movement_id: i64,
    pub ff3_transaction_id: i64,
}

impl From<NewMatch> for NewMatchModel {
    fn from(d: NewMatch) -> Self {
        Self {
            reconciliation_id: d.reconciliation_id.as_i64(),
            movement_id: d.movement_id.as_i64(),
            ff3_transaction_id: d.ff3_transaction_id.as_i64(),
        }
    }
}
