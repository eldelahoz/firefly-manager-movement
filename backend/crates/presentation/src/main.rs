use ffm_infrastructure::persistence::migrations::run_migrations;
use ffm_infrastructure::persistence::pool::create_pool;
use ffm_presentation::app;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL debe estar definida");
    let pool = create_pool(&database_url).expect("no se pudo crear el pool MySQL");
    run_migrations(&pool)
        .await
        .expect("no se pudieron ejecutar las migraciones");
    tracing::info!("migraciones aplicadas");

    let app = app();

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("ffm-backend escuchando en http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
