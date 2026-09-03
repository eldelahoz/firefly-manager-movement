//! Diesel models (tables) and conversions to/from domain entities.
//!
//! JSON fields are mapped to `serde_json::Value` in the model (diesel `Json`)
//! and converted to `String` in the domain (which treats JSON as text).
//! One file per aggregate (mirrors `domain::entities`), re-exported here so
//! the public path stays `ffm_infrastructure::models::*`.

mod app_settings;
mod batch;
mod creation;
mod csv_mapping;
mod deleted_audit;
mod ff3_account;
mod ff3_transaction;
mod match_model;
mod movement;
mod reconciliation;

pub use app_settings::{AppSettingsModel, NewAppSettingsModel};
pub use batch::{BatchModel, NewBatchModel};
pub use creation::{CreationModel, NewCreationModel};
pub use csv_mapping::{CsvMappingModel, NewCsvMappingModel};
pub use deleted_audit::{DeletedAuditModel, NewDeletedAuditModel};
pub use ff3_account::{Ff3AccountModel, NewFf3AccountModel};
pub use ff3_transaction::{Ff3TransactionModel, NewFf3TransactionModel};
pub use match_model::{MatchModel, NewMatchModel};
pub use movement::{MovementModel, NewMovementModel};
pub use reconciliation::{NewReconciliationModel, ReconciliationModel};
