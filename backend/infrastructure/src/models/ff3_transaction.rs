use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use ffm_application::dtos::NewFf3Transaction;
use ffm_domain::entities::Ff3Transaction;
use ffm_domain::ids::{Ff3TransactionId, ReconciliationId};
use ffm_domain::value_objects::AmountCents;

use crate::schema::ff3_transactions;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = ff3_transactions)]
pub struct Ff3TransactionModel {
    pub id: i64,
    pub reconciliation_id: i64,
    pub ff3_id: i64,
    pub ff3_journal_id: i64,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub is_duplicate: bool,
    pub created_at: NaiveDateTime,
}

impl From<Ff3TransactionModel> for Ff3Transaction {
    fn from(m: Ff3TransactionModel) -> Self {
        Self {
            id: Ff3TransactionId::new(m.id),
            reconciliation_id: ReconciliationId::new(m.reconciliation_id),
            ff3_id: m.ff3_id,
            ff3_journal_id: m.ff3_journal_id,
            date: m.date,
            amount_cents: AmountCents::new(m.amount_cents),
            description: m.description,
            is_duplicate: m.is_duplicate,
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = ff3_transactions)]
pub struct NewFf3TransactionModel {
    pub reconciliation_id: i64,
    pub ff3_id: i64,
    pub ff3_journal_id: i64,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub is_duplicate: bool,
}

impl From<NewFf3Transaction> for NewFf3TransactionModel {
    fn from(d: NewFf3Transaction) -> Self {
        Self {
            reconciliation_id: d.reconciliation_id.as_i64(),
            ff3_id: d.ff3_id,
            ff3_journal_id: 0,
            date: d.date,
            amount_cents: d.amount_cents.as_i64(),
            description: d.description,
            is_duplicate: d.is_duplicate,
        }
    }
}
