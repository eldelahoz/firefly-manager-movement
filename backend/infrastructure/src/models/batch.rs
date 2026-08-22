use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::prelude::*;

use ffm_application::dtos::NewBatch;
use ffm_domain::entities::Batch;
use ffm_domain::ids::{BatchId, Ff3AccountId};
use ffm_domain::value_objects::BatchStatus;

use crate::schema::batches;

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = batches)]
pub struct BatchModel {
    pub id: i64,
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<i64>,
    pub status: String,
    pub created_at: NaiveDateTime,
}

impl From<BatchModel> for Batch {
    fn from(m: BatchModel) -> Self {
        Self {
            id: BatchId::new(m.id),
            filename: m.filename,
            file_hash: m.file_hash,
            format: m.format,
            currency: m.currency,
            account_id: m.account_id.map(Ff3AccountId::new),
            status: BatchStatus::from_str(&m.status).unwrap_or(BatchStatus::Parsed),
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = batches)]
pub struct NewBatchModel {
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<i64>,
    pub status: String,
}

impl From<NewBatch> for NewBatchModel {
    fn from(d: NewBatch) -> Self {
        Self {
            filename: d.filename,
            file_hash: d.file_hash,
            format: d.format,
            currency: d.currency,
            account_id: d.account_id.map(Ff3AccountId::as_i64),
            status: d.status.as_str().to_string(),
        }
    }
}
