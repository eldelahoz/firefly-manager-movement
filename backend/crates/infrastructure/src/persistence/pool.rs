//! Pool de conexiones MySQL asíncronas (diesel-async + deadpool).

use diesel_async::pooled_connection::deadpool::{BuildError, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncMysqlConnection;

pub type DbPool = Pool<AsyncMysqlConnection>;

/// Crea un pool de conexiones MySQL a partir de la URL.
pub fn create_pool(database_url: &str) -> Result<DbPool, BuildError> {
    let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);
    Pool::builder(config).max_size(8).build()
}
