//! Helper for integration tests: creates the pool and cleans tables before each test.

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncMysqlConnection;
use diesel_async::RunQueryDsl;

use ffm_infrastructure::schema::{
    batches, creations, csv_mappings, deleted_audit, ff3_accounts, ff3_transactions, matches,
    movements, reconciliations,
};

pub type DbPool = Pool<AsyncMysqlConnection>;

/// `diesel::delete` on each table over the same connection (tables have
/// distinct types, so a heterogeneous array cannot be iterated).
macro_rules! truncate {
    ($conn:expr, $($table:expr),+ $(,)?) => {
        $(
            diesel::delete($table).execute($conn).await.expect("delete");
        )+
    };
}

pub async fn setup_pool() -> DbPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://ffm:devffmpass@127.0.0.1:3306/firefly_manager".to_string());
    let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(url);
    Pool::builder(config).max_size(4).build().expect("pool")
}

/// Truncates all tables (with FK checks disabled) for a clean state.
pub async fn clean(db: &DbPool) {
    let mut conn = db.get().await.expect("conn");
    diesel::sql_query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut conn)
        .await
        .expect("disable fk");
    truncate!(
        &mut conn,
        matches::table,
        creations::table,
        ff3_transactions::table,
        movements::table,
        reconciliations::table,
        batches::table,
        csv_mappings::table,
        deleted_audit::table,
        ff3_accounts::table,
    );
    diesel::sql_query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut conn)
        .await
        .expect("enable fk");
}
