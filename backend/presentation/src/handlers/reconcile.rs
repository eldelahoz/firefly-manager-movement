//! Rapprochement handlers (F4).

use axum::extract::{Path, State};
use axum::Json;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ffm_application::use_cases::get_results::GetResults;
use ffm_application::use_cases::ignore_movement::IgnoreMovement;
use ffm_application::use_cases::reconcile_batch::{ReconcileBatch, ReconcileBatchInput};

use ffm_domain::entities::{Ff3Transaction, Movement};
use ffm_domain::ids::{BatchId, MovementId};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct ReconcileRequest {
    /// Target FF3 asset account (external id).
    pub ff3_account_id: i64,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    #[serde(default)]
    pub tolerance_days: i32,
}

#[derive(Serialize, ToSchema)]
pub struct ReconcileResponse {
    pub reconciliation_id: i64,
    pub batch_id: i64,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: usize,
    pub missing_count: usize,
    /// Unmatched FF3 transactions: plain extra + duplicates.
    pub extra_count: usize,
    pub duplicate_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct Ff3TransactionResponse {
    pub id: i64,
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub is_duplicate: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct MatchedPairResponse {
    pub movement: MovementResponse,
    pub ff3: Ff3TransactionResponse,
}

#[derive(Serialize, ToSchema)]
pub struct MovementResponse {
    pub id: i64,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetResultsResponse {
    pub reconciliation_id: i64,
    pub matched: Vec<MatchedPairResponse>,
    pub missing: Vec<MovementResponse>,
    /// Unmatched FF3 transactions; duplicates carry `is_duplicate = true`.
    pub extra: Vec<Ff3TransactionResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct IgnoreMovementResponse {
    pub movement_id: i64,
    pub status: String,
}

impl From<Movement> for MovementResponse {
    fn from(m: Movement) -> Self {
        Self {
            id: m.id.as_i64(),
            line_no: m.line_no,
            date: m.date,
            amount_cents: m.amount_cents.as_i64(),
            description: m.description,
            status: m.status.as_str().to_string(),
        }
    }
}

impl From<Ff3Transaction> for Ff3TransactionResponse {
    fn from(t: Ff3Transaction) -> Self {
        Self {
            id: t.id.as_i64(),
            ff3_id: t.ff3_id,
            date: t.date,
            amount_cents: t.amount_cents.as_i64(),
            description: t.description,
            is_duplicate: t.is_duplicate,
            created_at: t.created_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/imports/{id}/reconcile",
    request_body = ReconcileRequest,
    params(("id" = i64, Path, description = "Batch id")),
    responses(
        (status = 200, description = "Rapprochement executed", body = ReconcileResponse),
        (status = 400, description = "Invalid window or currency mismatch"),
        (status = 404, description = "Batch or FF3 account not found"),
        (status = 502, description = "Firefly III unreachable"),
    ),
    tag = "reconcile",
)]
pub async fn reconcile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ReconcileRequest>,
) -> Result<Json<ReconcileResponse>, ApiError> {
    let uc = ReconcileBatch::new(
        state.firefly.clone(),
        state.ff3_accounts.clone(),
        state.batches.clone(),
        state.movements.clone(),
        state.reconciliations.clone(),
    );
    let result = uc
        .execute(ReconcileBatchInput {
            batch_id: BatchId::new(id),
            ff3_account_id: body.ff3_account_id,
            window_start: body.window_start,
            window_end: body.window_end,
            tolerance_days: body.tolerance_days,
        })
        .await
        .map_err(ApiError)?;

    Ok(Json(ReconcileResponse {
        reconciliation_id: result.reconciliation_id.as_i64(),
        batch_id: result.batch_id.as_i64(),
        tolerance_days: result.tolerance_days,
        window_start: result.window_start,
        window_end: result.window_end,
        matched_count: result.matched_count,
        missing_count: result.missing_count,
        extra_count: result.extra_count,
        duplicate_count: result.duplicate_count,
    }))
}

#[utoipa::path(
    get,
    path = "/api/imports/{id}/results",
    params(("id" = i64, Path, description = "Batch id")),
    responses(
        (status = 200, description = "Results in three groups", body = GetResultsResponse),
        (status = 404, description = "Batch has no reconciliation yet"),
    ),
    tag = "reconcile",
)]
pub async fn get_results(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<GetResultsResponse>, ApiError> {
    let uc = GetResults::new(
        state.reconciliations.clone(),
        state.matches.clone(),
        state.ff3_transactions.clone(),
        state.movements.clone(),
    );
    let result = uc.execute(BatchId::new(id)).await.map_err(ApiError)?;

    Ok(Json(GetResultsResponse {
        reconciliation_id: result.reconciliation_id.as_i64(),
        matched: result
            .matched
            .into_iter()
            .map(|p| MatchedPairResponse {
                movement: p.movement.into(),
                ff3: p.ff3.into(),
            })
            .collect(),
        missing: result.missing.into_iter().map(Into::into).collect(),
        extra: result.extra.into_iter().map(Into::into).collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/imports/{id}/movements/{mid}/ignore",
    params(
        ("id" = i64, Path, description = "Batch id"),
        ("mid" = i64, Path, description = "Movement id"),
    ),
    responses(
        (status = 200, description = "Movement ignored", body = IgnoreMovementResponse),
        (status = 404, description = "Batch or movement not found"),
    ),
    tag = "reconcile",
)]
pub async fn ignore_movement(
    State(state): State<AppState>,
    Path((id, mid)): Path<(i64, i64)>,
) -> Result<Json<IgnoreMovementResponse>, ApiError> {
    let uc = IgnoreMovement::new(state.movements.clone(), state.batches.clone());
    uc.execute(BatchId::new(id), MovementId::new(mid))
        .await
        .map_err(ApiError)?;

    Ok(Json(IgnoreMovementResponse {
        movement_id: mid,
        status: "ignored".to_string(),
    }))
}
