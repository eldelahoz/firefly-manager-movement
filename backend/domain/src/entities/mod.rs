//! Domain entities (POD structs).
//! No serde or diesel derives - those are infrastructure details.
//! One file per aggregate, re-exported here so the public path stays
//! `ffm_domain::entities::*`.

mod app_settings;
mod batch;
mod creation;
mod csv_mapping;
mod deleted_audit;
mod ff3_account;
mod ff3_transaction;
mod match_entity;
mod movement;
mod reconciliation;

pub use app_settings::AppSettings;
pub use batch::Batch;
pub use creation::Creation;
pub use csv_mapping::CsvMapping;
pub use deleted_audit::DeletedAudit;
pub use ff3_account::Ff3Account;
pub use ff3_transaction::Ff3Transaction;
pub use match_entity::Match;
pub use movement::Movement;
pub use reconciliation::Reconciliation;
