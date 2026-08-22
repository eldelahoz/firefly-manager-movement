//! Embedded migrations — run at backend startup (no `diesel_cli` needed at runtime).

use diesel_async::AsyncMigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use ffm_application::error::ApplicationError;

use crate::persistence::pool::DbPool;

/// Migrations embedded into the binary (path relative to this crate's CARGO_MANIFEST_DIR).
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Runs pending migrations against the given pool.
/// Called at backend boot (in `presentation::main`).
pub async fn run_migrations(pool: &DbPool) -> Result<(), ApplicationError> {
    let conn = pool
        .get()
        .await
        .map_err(|e| ApplicationError::Database(e.to_string()))?;
    let mut harness = AsyncMigrationHarness::new(conn);
    harness
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| ApplicationError::Database(e.to_string()))?;
    Ok(())
}
