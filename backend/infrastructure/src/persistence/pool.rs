//! Async SQLite connection pool (diesel-async `SyncConnectionWrapper` + deadpool).

use diesel::{ConnectionError, SqliteConnection};
use diesel_async::pooled_connection::deadpool::{BuildError, Pool};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, SimpleAsyncConnection};

pub type DbConnection = SyncConnectionWrapper<SqliteConnection>;
pub type DbPool = Pool<DbConnection>;

/// Creates a pool on the SQLite file at `database_url` (created if missing).
pub fn create_pool(database_url: &str) -> Result<DbPool, BuildError> {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(|url| Box::pin(establish(url.to_owned())));
    let manager = AsyncDieselConnectionManager::new_with_config(database_url, config);
    Pool::builder(manager).max_size(8).build()
}

async fn establish(url: String) -> Result<DbConnection, ConnectionError> {
    let mut conn = DbConnection::establish(&url).await?;
    // SQLite has one writer: WAL + busy_timeout avoid SQLITE_BUSY under the pool; FKs are off by default.
    conn.batch_execute(
        "PRAGMA busy_timeout = 5000; PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;",
    )
    .await
    .map_err(ConnectionError::CouldntSetupConfiguration)?;
    Ok(conn)
}
