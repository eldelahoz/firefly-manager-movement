use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use ffm_application::dtos::NewReconciliation;
use ffm_domain::entities::Reconciliation;
use ffm_domain::ids::{BatchId, ReconciliationId};

use crate::schema::reconciliations;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = reconciliations)]
pub struct ReconciliationModel {
    pub id: i64,
    pub batch_id: i64,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
    pub ran_at: NaiveDateTime,
}

impl From<ReconciliationModel> for Reconciliation {
    fn from(m: ReconciliationModel) -> Self {
        Self {
            id: ReconciliationId::new(m.id),
            batch_id: BatchId::new(m.batch_id),
            tolerance_days: m.tolerance_days,
            window_start: m.window_start,
            window_end: m.window_end,
            matched_count: m.matched_count,
            missing_count: m.missing_count,
            extra_count: m.extra_count,
            ran_at: m.ran_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = reconciliations)]
pub struct NewReconciliationModel {
    pub batch_id: i64,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
}

impl From<NewReconciliation> for NewReconciliationModel {
    fn from(d: NewReconciliation) -> Self {
        Self {
            batch_id: d.batch_id.as_i64(),
            tolerance_days: d.tolerance_days,
            window_start: d.window_start,
            window_end: d.window_end,
            matched_count: d.matched_count,
            missing_count: d.missing_count,
            extra_count: d.extra_count,
        }
    }
}
