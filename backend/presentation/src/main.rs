use std::env;
use std::sync::Arc;

use ffm_infrastructure::csv::EncodingCsvDecoder;
use ffm_infrastructure::firefly::client::ReqwestFireflyClient;
use ffm_infrastructure::persistence::migrations::run_migrations;
use ffm_infrastructure::persistence::pool::create_pool;
use ffm_infrastructure::persistence::repositories::batch::MysqlBatchRepository;
use ffm_infrastructure::persistence::repositories::csv_mapping::MysqlCsvMappingRepository;
use ffm_infrastructure::persistence::repositories::ff3_account::MysqlFf3AccountRepository;
use ffm_infrastructure::persistence::repositories::movement::MysqlMovementRepository;
use ffm_presentation::{app, AppState};

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
    let pool = create_pool(&database_url).expect("could not create MySQL pool");
    run_migrations(&pool)
        .await
        .expect("could not run migrations");
    tracing::info!("migrations applied");

    let ff3_url = env::var("FF3_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let ff3_token = env::var("FF3_TOKEN").expect("FF3_TOKEN must be set");
    let ff3_health_probe = env::var("FF3_HEALTH_PROBE")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let firefly = Arc::new(ReqwestFireflyClient::new(&ff3_url, &ff3_token));
    let ff3_accounts = Arc::new(MysqlFf3AccountRepository::new(pool.clone()));
    let csv_decoder = Arc::new(EncodingCsvDecoder);
    let batches = Arc::new(MysqlBatchRepository::new(pool.clone()));
    let movements = Arc::new(MysqlMovementRepository::new(pool.clone()));
    let csv_mappings = Arc::new(MysqlCsvMappingRepository::new(pool.clone()));

    let state = AppState {
        firefly,
        ff3_accounts,
        csv_decoder,
        batches,
        movements,
        csv_mappings,
        ff3_health_probe,
    };

    let app = app(state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("ffm-backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
