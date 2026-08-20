//! Entidades del dominio (POD structs).
//! Sin derives de serde ni diesel — esos son detalles de infraestructura.

use chrono::{NaiveDate, NaiveDateTime};

use crate::ids::{
    BatchId, CreationId, CsvMappingId, DeletedAuditId, Ff3AccountId, Ff3TransactionId, MatchId,
    MovementId, ReconciliationId,
};
use crate::value_objects::{AmountCents, BatchStatus, HeaderSignature, MovementStatus};

/// Caché local de una cuenta de activo de Firefly III.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff3Account {
    pub id: Ff3AccountId,
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub synced_at: NaiveDateTime,
}

/// Lote de movimientos importados de un archivo (inmutable, RF-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub id: BatchId,
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<Ff3AccountId>,
    pub status: BatchStatus,
    pub created_at: NaiveDateTime,
}

/// Movimiento importado de un relevé bancario.
/// `amount_cents` es con signo (negativo = débito); sin columna `direction` (TRD D2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    pub id: MovementId,
    pub batch_id: BatchId,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub status: MovementStatus,
    pub created_at: NaiveDateTime,
}

/// Resultado congelado de un run de rapprochement (audit snapshot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    pub id: ReconciliationId,
    pub batch_id: BatchId,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
    pub ran_at: NaiveDateTime,
}

/// Transacción de Firefly III examinada en un run (caché por run).
/// `is_duplicate` marca las tx FF3 surnuméraires face a un mismo movimiento (RF-15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff3Transaction {
    pub id: Ff3TransactionId,
    pub reconciliation_id: ReconciliationId,
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub is_duplicate: bool,
    pub created_at: NaiveDateTime,
}

/// Emparejamiento 1-a-1 historizado por run (el 1-a-1 lo garantiza el schema).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub id: MatchId,
    pub reconciliation_id: ReconciliationId,
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
    pub matched_at: NaiveDateTime,
}

/// Trazabilidad de una transacción creada por nosotros en Firefly III.
/// `UNIQUE(movement_id)` → idempotencia (NFR-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Creation {
    pub id: CreationId,
    pub movement_id: MovementId,
    pub ff3_id: i64,
    pub created_at: NaiveDateTime,
}

/// Mapping CSV memorizado por firma de encabezados (RF-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvMapping {
    pub id: CsvMappingId,
    pub header_signature: HeaderSignature,
    pub mapping_json: String,
    pub created_at: NaiveDateTime,
}

/// Snapshot de auditoría de una transacción FF3 eliminada (point-in-time, denormalizado).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedAudit {
    pub id: DeletedAuditId,
    pub ff3_id: i64,
    pub snapshot_json: String,
    pub deleted_at: NaiveDateTime,
}
