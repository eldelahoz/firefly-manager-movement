//! ffm-presentation — capa HTTP (axum + utoipa/Scalar).

use axum::response::Html;
use axum::routing::get;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::Scalar;

/// Respuesta del health check.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Backend healthy", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

/// Construye el router axum con los endpoints + Scalar + spec JSON.
pub fn app() -> axum::Router {
    // 1) Router de endpoints documentados + spec OpenAPI resultante.
    let (router, openapi) = OpenApiRouter::new()
        .routes(routes!(health))
        .split_for_parts();

    // 2) Pre-serializamos la spec a un Value (Clone + Send + Sync) para servir el JSON
    //    sin necesitar que OpenApi: Clone.
    let openapi_value =
        serde_json::to_value(&openapi).expect("la spec OpenAPI debe serializarse");

    // 3) Scalar standalone: HTML pre-renderado (sin feature "axum" → evita acoplamiento
    //    de versión de axum entre utoipa-scalar y nuestro axum 0.8).
    let scalar_html = Scalar::new(openapi).to_html();

    router
        .route("/scalar", get(move || async move { Html(scalar_html) }))
        .route(
            "/api-docs/openapi.json",
            get(move || async move { Json(openapi_value) }),
        )
}