//! ffm-presentation — HTTP layer (axum + utoipa/Scalar).

use axum::response::Html;
use axum::routing::get;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::Scalar;

/// Health check response.
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

/// Builds the axum router with the endpoints + Scalar + JSON spec.
pub fn app() -> axum::Router {
    // 1) Router of documented endpoints + resulting OpenAPI spec.
    let (router, openapi) = OpenApiRouter::new()
        .routes(routes!(health))
        .split_for_parts();

    // 2) Pre-serialize the spec to a Value (Clone + Send + Sync) to serve the JSON
    //    without requiring OpenApi: Clone.
    let openapi_value = serde_json::to_value(&openapi).expect("la spec OpenAPI debe serializarse");

    // 3) Standalone Scalar: pre-rendered HTML (no "axum" feature → avoids axum version
    //    coupling between utoipa-scalar and our axum 0.8).
    let scalar_html = Scalar::new(openapi).to_html();

    router
        .route("/scalar", get(move || async move { Html(scalar_html) }))
        .route(
            "/api-docs/openapi.json",
            get(move || async move { Json(openapi_value) }),
        )
}
