//! Firefly III proxy handlers.

use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::state::AppState;

use ffm_application::error::ApplicationError;
use ffm_application::use_cases::get_account_transactions::GetAccountTransactions;
use ffm_application::use_cases::list_firefly_accounts::ListFireflyAccounts;

// ── Request / Response DTOs ─────────────────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct PeriodRequest {
    #[schema(example = "2026-01-01")]
    pub start: NaiveDate,
    #[schema(example = "2026-01-31")]
    pub end: NaiveDate,
}

#[derive(Serialize, ToSchema)]
pub struct FireflyAccountResponse {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency: String,
}

#[derive(Serialize, ToSchema)]
pub struct FireflyTransactionResponse {
    pub id: i64,
    #[schema(example = "2026-01-05")]
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
}

// ── GET /api/firefly/accounts ───────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/firefly/accounts",
    responses(
        (status = 200, description = "List of FF3 asset accounts (synced to local cache)", body = [FireflyAccountResponse]),
        (status = 502, description = "Firefly III unreachable"),
    ),
    tag = "firefly",
)]
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<FireflyAccountResponse>>, ApiError> {
    let uc = ListFireflyAccounts::new(state.firefly.clone(), state.ff3_accounts.clone());
    let accounts = uc.execute().await.map_err(ApiError)?;
    Ok(Json(
        accounts
            .into_iter()
            .map(|a| FireflyAccountResponse {
                id: a.ff3_id,
                name: a.name,
                account_type: a.account_type,
                currency: a.currency,
            })
            .collect(),
    ))
}

// ── QUERY / POST /api/firefly/accounts/{id}/transactions ────────────────

#[utoipa::path(
    post,
    path = "/api/firefly/accounts/{id}/transactions",
    request_body = PeriodRequest,
    responses(
        (status = 200, description = "Transactions for the account in the period", body = [FireflyTransactionResponse]),
        (status = 400, description = "Invalid request (start > end, bad dates)"),
        (status = 502, description = "Firefly III unreachable"),
    ),
    tag = "firefly",
    description = "Get account transactions for a period. Also accepts RFC 9295 QUERY method.",
)]
pub async fn account_transactions(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PeriodRequest>,
) -> Result<Json<Vec<FireflyTransactionResponse>>, ApiError> {
    handle_transactions(state, id, body).await
}

/// Fallback handler: serves QUERY method, rejects others with 405.
pub async fn account_transactions_fallback(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    request: Request,
) -> impl IntoResponse {
    if request.method().as_str() != "QUERY" {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(crate::error::ErrorBody {
                error: "method not allowed — use POST or QUERY".into(),
            }),
        )
            .into_response();
    }
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            return ApiError(ApplicationError::Parse(format!("failed to read body: {e}")))
                .into_response()
        }
    };
    let period: PeriodRequest = match serde_json::from_slice(&body_bytes) {
        Ok(p) => p,
        Err(e) => {
            return ApiError(ApplicationError::Parse(format!("invalid JSON body: {e}")))
                .into_response()
        }
    };
    match handle_transactions(state, id, period).await {
        Ok(json) => json.into_response(),
        Err(e) => e.into_response(),
    }
}

async fn handle_transactions(
    state: AppState,
    id: i64,
    body: PeriodRequest,
) -> Result<Json<Vec<FireflyTransactionResponse>>, ApiError> {
    let uc = GetAccountTransactions::new(state.firefly.clone());
    let txs = uc
        .execute(id, body.start, body.end)
        .await
        .map_err(ApiError)?;
    Ok(Json(
        txs.into_iter()
            .map(|t| FireflyTransactionResponse {
                id: t.ff3_id,
                date: t.date,
                amount_cents: t.amount_cents.as_i64(),
                description: t.description,
            })
            .collect(),
    ))
}
