//! Modelos Diesel (tablas) y conversiones hacia/desde las entidades del dominio.
//!
//! Los campos JSON se mapean a `serde_json::Value` en el modelo (diesel `Json`)
//! y se convierten a `String` en el dominio (que trata el JSON como texto).

use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde_json::Value as JsonValue;
use std::str::FromStr;

use ffm_application::dtos::{
    NewBatch, NewCreation, NewCsvMapping, NewDeletedAudit, NewFf3Account, NewFf3Transaction,
    NewMatch, NewMovement, NewReconciliation,
};
use ffm_domain::entities::{
    Batch, Creation, CsvMapping, DeletedAudit, Ff3Account, Ff3Transaction, Match, Movement,
    Reconciliation,
};
use ffm_domain::ids::*;
use ffm_domain::value_objects::{AmountCents, BatchStatus, HeaderSignature, MovementStatus};

use crate::schema::{
    batches, creations, csv_mappings, deleted_audit, ff3_accounts, ff3_transactions, matches,
    movements, reconciliations,
};

// ============================ ff3_accounts ============================

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = ff3_accounts)]
pub struct Ff3AccountModel {
    pub id: i64,
    pub ff3_id: i32,
    pub name: String,
    pub type_: String,
    pub currency: String,
    pub synced_at: NaiveDateTime,
}

impl From<Ff3AccountModel> for Ff3Account {
    fn from(m: Ff3AccountModel) -> Self {
        Self {
            id: Ff3AccountId::new(m.id),
            ff3_id: m.ff3_id as i64,
            name: m.name,
            account_type: m.type_,
            currency: m.currency,
            synced_at: m.synced_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = ff3_accounts)]
pub struct NewFf3AccountModel {
    pub ff3_id: i32,
    pub name: String,
    pub type_: String,
    pub currency: String,
}

impl From<NewFf3Account> for NewFf3AccountModel {
    fn from(d: NewFf3Account) -> Self {
        Self {
            ff3_id: d.ff3_id as i32,
            name: d.name,
            type_: d.account_type,
            currency: d.currency,
        }
    }
}

// ============================ batches ============================

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

// ============================ movements ============================

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = movements)]
pub struct MovementModel {
    pub id: i64,
    pub batch_id: i64,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub raw_json: JsonValue,
    pub status: String,
    pub created_at: NaiveDateTime,
}

impl From<MovementModel> for Movement {
    fn from(m: MovementModel) -> Self {
        Self {
            id: MovementId::new(m.id),
            batch_id: BatchId::new(m.batch_id),
            line_no: m.line_no,
            date: m.date,
            amount_cents: AmountCents::new(m.amount_cents),
            description: m.description,
            raw_json: m.raw_json.to_string(),
            status: MovementStatus::from_str(&m.status).unwrap_or(MovementStatus::Imported),
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = movements)]
pub struct NewMovementModel {
    pub batch_id: i64,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub raw_json: JsonValue,
    pub status: String,
}

impl From<NewMovement> for NewMovementModel {
    fn from(d: NewMovement) -> Self {
        Self {
            batch_id: d.batch_id.as_i64(),
            line_no: d.line_no,
            date: d.date,
            amount_cents: d.amount_cents.as_i64(),
            description: d.description,
            raw_json: serde_json::from_str(&d.raw_json).unwrap_or(JsonValue::Null),
            status: d.status.as_str().to_string(),
        }
    }
}

// ============================ reconciliations ============================

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

// ============================ ff3_transactions ============================

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = ff3_transactions)]
pub struct Ff3TransactionModel {
    pub id: i64,
    pub reconciliation_id: i64,
    pub ff3_id: i64,
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
            date: d.date,
            amount_cents: d.amount_cents.as_i64(),
            description: d.description,
            is_duplicate: d.is_duplicate,
        }
    }
}

// ============================ matches ============================

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

// ============================ creations ============================

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = creations)]
pub struct CreationModel {
    pub id: i64,
    pub movement_id: i64,
    pub ff3_id: i64,
    pub created_at: NaiveDateTime,
}

impl From<CreationModel> for Creation {
    fn from(m: CreationModel) -> Self {
        Self {
            id: CreationId::new(m.id),
            movement_id: MovementId::new(m.movement_id),
            ff3_id: m.ff3_id,
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = creations)]
pub struct NewCreationModel {
    pub movement_id: i64,
    pub ff3_id: i64,
}

impl From<NewCreation> for NewCreationModel {
    fn from(d: NewCreation) -> Self {
        Self {
            movement_id: d.movement_id.as_i64(),
            ff3_id: d.ff3_id,
        }
    }
}

// ============================ csv_mappings ============================

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = csv_mappings)]
pub struct CsvMappingModel {
    pub id: i64,
    pub header_signature: String,
    pub mapping_json: JsonValue,
    pub created_at: NaiveDateTime,
}

impl From<CsvMappingModel> for CsvMapping {
    fn from(m: CsvMappingModel) -> Self {
        Self {
            id: CsvMappingId::new(m.id),
            header_signature: HeaderSignature::from_str(&m.header_signature).unwrap(),
            mapping_json: m.mapping_json.to_string(),
            created_at: m.created_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = csv_mappings)]
pub struct NewCsvMappingModel {
    pub header_signature: String,
    pub mapping_json: JsonValue,
}

impl From<NewCsvMapping> for NewCsvMappingModel {
    fn from(d: NewCsvMapping) -> Self {
        Self {
            header_signature: d.header_signature.as_str().to_string(),
            mapping_json: serde_json::from_str(&d.mapping_json).unwrap_or(JsonValue::Null),
        }
    }
}

// ============================ deleted_audit ============================

#[derive(Queryable, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = deleted_audit)]
pub struct DeletedAuditModel {
    pub id: i64,
    pub ff3_id: i64,
    pub snapshot_json: JsonValue,
    pub deleted_at: NaiveDateTime,
}

impl From<DeletedAuditModel> for DeletedAudit {
    fn from(m: DeletedAuditModel) -> Self {
        Self {
            id: DeletedAuditId::new(m.id),
            ff3_id: m.ff3_id,
            snapshot_json: m.snapshot_json.to_string(),
            deleted_at: m.deleted_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = deleted_audit)]
pub struct NewDeletedAuditModel {
    pub ff3_id: i64,
    pub snapshot_json: JsonValue,
}

impl From<NewDeletedAudit> for NewDeletedAuditModel {
    fn from(d: NewDeletedAudit) -> Self {
        Self {
            ff3_id: d.ff3_id,
            snapshot_json: serde_json::from_str(&d.snapshot_json).unwrap_or(JsonValue::Null),
        }
    }
}
