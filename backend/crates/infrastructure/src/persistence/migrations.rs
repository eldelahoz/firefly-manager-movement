//! Migraciones embebidas — se ejecutan al arrancar el backend (no requiere `diesel_cli` en runtime).

use diesel_async::AsyncMigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use ffm_application::error::ApplicationError;

use crate::persistence::pool::DbPool;

/// Migraciones embebidas al binario (ruta relativa al CARGO_MANIFEST_DIR de este crate).
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");

/// Ejecuta las migraciones pendientes contra el pool dado.
/// Se llama al boot del backend (en `presentation::main`).
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
