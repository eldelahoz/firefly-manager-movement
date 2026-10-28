//! Application layer DTOs: creation types (inputs to the
//! repositories) and Firefly III client DTOs (outputs of the
//! `FireflyClient` port, external API shape - no local IDs).
//! One file per bounded context, re-exported here so the public path
//! stays `ffm_application::dtos::*`.

mod apply;
mod audit;
mod firefly;
mod imports;
mod reconciliation;
mod settings;

pub use apply::{ApplyMissingInput, AttachmentPayload, MovementApply};
pub use audit::NewDeletedAudit;
pub use firefly::{
    About, BrowseTxDto, Ff3AccountDto, Ff3SplitDto, Ff3TransactionDto, Ff3TxStatus, NewFf3Account,
    NewFf3TransactionCreation,
};
pub use imports::{NewBatch, NewCsvMapping, NewMovement};
pub use reconciliation::{
    NewCreation, NewFf3Transaction, NewMatch, NewReconciliation, ReconciliationRun,
    RunFf3Transaction, RunMatch,
};
pub use settings::{NewAppSettings, SettingsView};
