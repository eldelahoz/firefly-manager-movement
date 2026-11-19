//! Helper for integration tests: a fresh, migrated SQLite file per test.

use std::sync::atomic::{AtomicUsize, Ordering};

pub use ffm_infrastructure::persistence::pool::DbPool;
use ffm_infrastructure::persistence::{migrations::run_migrations, pool::create_pool};

/// Each call gets its own DB file, so tests stay isolated while running in parallel.
pub async fn setup_pool() -> DbPool {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "ffm-test-{}-{}.db",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    // shortcut: files are left in the temp dir after the run; add cleanup if they pile up.
    let _ = std::fs::remove_file(&path);
    let pool = create_pool(path.to_str().expect("utf-8 temp path")).expect("pool");
    run_migrations(&pool).await.expect("migrations");
    pool
}
