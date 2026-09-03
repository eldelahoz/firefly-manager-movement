//! Application layer ports (traits).
//!
//! Infrastructure adapters implement these traits. Use cases depend
//! on the interfaces, not on concrete implementations
//! (Dependency Inversion).
//! One file per bounded context, re-exported here so the public path
//! stays `ffm_application::ports::*`.

mod arc_impls;

mod audit;
mod clock;
mod firefly;
mod imports;
mod reconciliation;
mod settings;

pub use audit::DeletedAuditRepository;
pub use clock::Clock;
pub use firefly::{Ff3AccountRepository, FireflyClient};
pub use imports::{BatchRepository, CsvDecoder, CsvMappingRepository, MovementRepository};
pub use reconciliation::{
    CreationRepository, Ff3TransactionRepository, MatchRepository, ReconciliationRepository,
};
pub use settings::{AppSettingsRepository, FireflyClientFactory, FireflyConnectionManager};
