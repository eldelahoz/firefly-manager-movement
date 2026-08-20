//! DTOs de la capa de aplicación: tipos de creación (entradas a los
//! repositorios) y DTOs del cliente Firefly III (salidas del puerto
//! `FireflyClient`, shape de la API externa — sin IDs locales).

use chrono::NaiveDate;

use ffm_domain::ids::{BatchId, Ff3AccountId, Ff3TransactionId, MovementId, ReconciliationId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, HeaderSignature, MovementStatus};

// ---- Tipos de creación (entradas a `*Repository::insert*`) ----
// Sin `id` ni timestamps (asignados por la DB / Clock en infra).

#[derive(Debug, Clone)]
pub struct NewBatch {
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<Ff3AccountId>,
    pub status: BatchStatus,
}

#[derive(Debug, Clone)]
pub struct NewMovement {
    pub batch_id: BatchId,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub status: MovementStatus,
}

#[derive(Debug, Clone)]
pub struct NewReconciliation {
    pub batch_id: BatchId,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
}

#[derive(Debug, Clone)]
pub struct NewFf3Transaction {
    pub reconciliation_id: ReconciliationId,
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub is_duplicate: bool,
}

#[derive(Debug, Clone)]
pub struct NewMatch {
    pub reconciliation_id: ReconciliationId,
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
}

#[derive(Debug, Clone)]
pub struct NewCreation {
    pub movement_id: MovementId,
    pub ff3_id: i64,
}

#[derive(Debug, Clone)]
pub struct NewCsvMapping {
    pub header_signature: HeaderSignature,
    pub mapping_json: String,
}

#[derive(Debug, Clone)]
pub struct NewDeletedAudit {
    pub ff3_id: i64,
    pub snapshot_json: String,
}

#[derive(Debug, Clone)]
pub struct NewFf3Account {
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
}

// ---- DTOs del cliente Firefly III (shape de la API externa) ----

/// Respuesta de `GET /api/v1/about` (salud de FF3).
#[derive(Debug, Clone)]
pub struct About {
    pub version: String,
}

/// Cuenta de activo tal como la devuelve la API de Firefly III (sin ID local).
#[derive(Debug, Clone)]
pub struct Ff3AccountDto {
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
}

/// Transacción de Firefly III tal como la devuelve la API (sin ID local).
#[derive(Debug, Clone)]
pub struct Ff3TransactionDto {
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
}

/// Datos para crear una transacción en Firefly III (RF-08/RF-11).
#[derive(Debug, Clone)]
pub struct NewFf3TransactionCreation {
    pub transaction_type: String,
    pub amount: AmountCents,
    pub date: NaiveDate,
    pub description: String,
    pub source_ff3_id: i64,
    pub destination_ff3_id: i64,
    pub category: Option<String>,
    pub budget: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}
