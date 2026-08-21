//! Async MySQL connection pool (diesel-async + deadpool).

use diesel_async::pooled_connection::deadpool::{BuildError, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncMysqlConnection;

pub type DbPool = Pool<AsyncMysqlConnection>;

/// Creates a MySQL connection pool from the URL.
pub fn create_pool(database_url: &str) -> Result<DbPool, BuildError> {
    let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);
    Pool::builder(config).max_size(8).build()
}
