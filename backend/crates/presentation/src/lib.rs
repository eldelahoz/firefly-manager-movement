//! ffm-presentation — HTTP layer (axum + utoipa/Scalar).

use axum::extract::State;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::Scalar;

use crate::handlers::firefly::{
    __path_list_accounts, account_transactions, account_transactions_fallback, list_accounts,
};

mod error;
mod handlers;
mod state;

pub use error::{ApiError, ErrorBody};
pub use state::AppState;

/// Health check response.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ff3: Option<Ff3Health>,
}

#[derive(Serialize, ToSchema)]
pub struct Ff3Health {
    pub reachable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Backend healthy", body = HealthResponse),
        (status = 503, description = "Backend degraded (FF3 unreachable)", body = HealthResponse),
    ),
)]
async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    if !state.ff3_health_probe {
        return Json(HealthResponse {
            status: "ok".to_string(),
            ff3: None,
        });
    }
    match state.firefly.about().await {
        Ok(about) => Json(HealthResponse {
            status: "ok".to_string(),
            ff3: Some(Ff3Health {
                reachable: true,
                version: Some(about.version),
            }),
        }),
        Err(_) => Json(HealthResponse {
            status: "degraded".to_string(),
            ff3: Some(Ff3Health {
                reachable: false,
                version: None,
            }),
        }),
    }
}

/// Builds the axum router with the endpoints + Scalar + JSON spec.
pub fn app(state: AppState) -> axum::Router {
    // Transactions route: POST (documented) + QUERY via MethodRouter fallback.
    let tx_method_router = post(account_transactions).fallback(account_transactions_fallback);

    // 1) Router of documented endpoints + resulting OpenAPI spec.
    let (router, openapi) = OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(list_accounts))
        .route("/api/firefly/accounts/{id}/transactions", tx_method_router)
        .with_state(state)
        .split_for_parts();

    // 2) Pre-serialize the spec to a Value (Clone + Send + Sync) to serve the JSON
    //    without requiring OpenApi: Clone.
    let openapi_value = serde_json::to_value(&openapi).expect("OpenAPI spec must serialize");

    // 3) Standalone Scalar: pre-rendered HTML.
    let scalar_html = Scalar::new(openapi).to_html();

    router
        .route("/scalar", get(move || async move { Html(scalar_html) }))
        .route(
            "/api-docs/openapi.json",
            get(move || async move { Json(openapi_value) }),
        )
}
