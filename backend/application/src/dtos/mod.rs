//! Application layer DTOs: creation types (inputs to the
//! repositories) and Firefly III client DTOs (outputs of the
//! `FireflyClient` port, external API shape — no local IDs).
//! One file per bounded context, re-exported here so the public path
//! stays `ffm_application::dtos::*`.

mod audit;
mod firefly;
mod imports;
mod reconciliation;

pub use audit::NewDeletedAudit;
pub use firefly::{
    About, Ff3AccountDto, Ff3TransactionDto, NewFf3Account, NewFf3TransactionCreation,
};
pub use imports::{NewBatch, NewCsvMapping, NewMovement};
pub use reconciliation::{NewCreation, NewFf3Transaction, NewMatch, NewReconciliation};
