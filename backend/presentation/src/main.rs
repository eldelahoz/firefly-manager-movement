use std::env;
use std::sync::Arc;

use ffm_application::ports::AppSettingsRepository;
use ffm_infrastructure::csv::EncodingCsvDecoder;
use ffm_infrastructure::firefly::dynamic::{DynamicFireflyClient, ReqwestFireflyClientFactory};
use ffm_infrastructure::persistence::migrations::run_migrations;
use ffm_infrastructure::persistence::pool::create_pool;
use ffm_infrastructure::persistence::repositories::app_settings::SqliteAppSettingsRepository;
use ffm_infrastructure::persistence::repositories::batch::SqliteBatchRepository;
use ffm_infrastructure::persistence::repositories::creation::SqliteCreationRepository;
use ffm_infrastructure::persistence::repositories::csv_mapping::SqliteCsvMappingRepository;
use ffm_infrastructure::persistence::repositories::deleted_audit::SqliteDeletedAuditRepository;
use ffm_infrastructure::persistence::repositories::ff3_account::SqliteFf3AccountRepository;
use ffm_infrastructure::persistence::repositories::ff3_transaction::SqliteFf3TransactionRepository;
use ffm_infrastructure::persistence::repositories::match_repo::SqliteMatchRepository;
use ffm_infrastructure::persistence::repositories::movement::SqliteMovementRepository;
use ffm_infrastructure::persistence::repositories::reconciliation::SqliteReconciliationRepository;
use ffm_presentation::{app, serve_frontend, AppState};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).expect("could not create SQLite pool");
    run_migrations(&pool)
        .await
        .expect("could not run migrations");
    tracing::info!("migrations applied");

    let ff3_health_probe = env::var("FF3_HEALTH_PROBE")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    // Settings-driven connection (RF-19): read from DB, fall back to env vars.
    let app_settings = Arc::new(SqliteAppSettingsRepository::new(pool.clone()));
    let ff3_defaults = (
        env::var("FF3_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
        env::var("FF3_TOKEN").unwrap_or_default(),
    );
    let firefly: Arc<DynamicFireflyClient> = Arc::new(
        match app_settings
            .get()
            .await
            .expect("could not read app settings")
        {
            Some(s) => DynamicFireflyClient::new(&s.ff3_url, &s.ff3_token),
            None => DynamicFireflyClient::new(&ff3_defaults.0, &ff3_defaults.1),
        },
    );
    let firefly_manager: Arc<dyn ffm_application::ports::FireflyConnectionManager> =
        firefly.clone();
    let firefly_factory: Arc<dyn ffm_application::ports::FireflyClientFactory> =
        Arc::new(ReqwestFireflyClientFactory);

    let ff3_accounts = Arc::new(SqliteFf3AccountRepository::new(pool.clone()));
    let csv_decoder = Arc::new(EncodingCsvDecoder);
    let batches = Arc::new(SqliteBatchRepository::new(pool.clone()));
    let movements = Arc::new(SqliteMovementRepository::new(pool.clone()));
    let csv_mappings = Arc::new(SqliteCsvMappingRepository::new(pool.clone()));
    let reconciliations = Arc::new(SqliteReconciliationRepository::new(pool.clone()));
    let matches = Arc::new(SqliteMatchRepository::new(pool.clone()));
    let ff3_transactions = Arc::new(SqliteFf3TransactionRepository::new(pool.clone()));
    let creations = Arc::new(SqliteCreationRepository::new(pool.clone()));
    let deleted_audits = Arc::new(SqliteDeletedAuditRepository::new(pool.clone()));

    let state = AppState {
        firefly: firefly.clone(),
        ff3_accounts,
        csv_decoder,
        batches,
        movements,
        csv_mappings,
        reconciliations,
        matches,
        ff3_transactions,
        creations,
        deleted_audits,
        ff3_health_probe,
        app_settings,
        firefly_manager,
        firefly_factory,
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults {
            ff3_url: ff3_defaults.0,
            ff3_token: ff3_defaults.1,
        },
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        apply_lock: Arc::new(tokio::sync::Mutex::new(())),
    };

    let app = match env::var("FRONTEND_DIST") {
        Ok(dist) => serve_frontend(app(state), std::path::Path::new(&dist)),
        Err(_) => app(state),
    };

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("ffm-backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
