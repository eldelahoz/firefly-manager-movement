//! Rapprochement handlers (F4) + apply (F5).

use axum::extract::{Path, State};
use axum::Json;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ffm_application::dtos::{
    ApplyMissingInput, AttachmentPayload, MovementApply, SuggestedCounterparty,
};
use ffm_application::use_cases::apply_missing::{ApplyMissing, ApplyOutcome, RunPorts};
use ffm_application::use_cases::get_results::{GetResults, SuggestionPorts};
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
    /// Unmatched FF3 transactions: `extra` + `duplicates` (RF-07 third group).
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
    /// Opposite-side account name; null for snapshots saved before RF-07 v0.11.
    #[schema(example = "Acme Motors")]
    pub counterparty_name: Option<String>,
    /// Firefly III type of that account (`Expense account`, `Loan`, …).
    #[schema(example = "Expense account")]
    pub counterparty_type: Option<String>,
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

/// Suggested counterparty of a missing movement (RF-08 v0.11). Exactly one of
/// `ff3_id` (existing account, with `name` = its name as stored), a `name`
/// without id (a learned new name whose id Firefly III did not return) or
/// `cash: true` defines the choice. Ids are returned as stored: if that
/// account no longer exists, apply rejects it.
#[derive(Serialize, ToSchema)]
pub struct SuggestedCounterpartyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5)]
    pub ff3_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Acme Motors")]
    pub name: Option<String>,
    pub cash: bool,
    /// `learned` (chosen before in this app) or `history` (Firefly III history).
    #[schema(example = "learned")]
    pub source: String,
}

impl From<SuggestedCounterparty> for SuggestedCounterpartyResponse {
    fn from(s: SuggestedCounterparty) -> Self {
        Self {
            ff3_id: s.counterparty.ff3_id,
            name: s.counterparty.name,
            cash: s.counterparty.cash,
            source: s.source.as_str().to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct MissingMovementResponse {
    #[serde(flatten)]
    pub movement: MovementResponse,
    /// Null when there is nothing to suggest. Learned choices are resolved
    /// when the results are read; history suggestions are stored at reconcile
    /// time, so batches reconciled before RF-08 v0.11 have none until they are
    /// reconciled again.
    pub suggested_counterparty: Option<SuggestedCounterpartyResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct GetResultsResponse {
    pub reconciliation_id: i64,
    pub matched: Vec<MatchedPairResponse>,
    pub missing: Vec<MissingMovementResponse>,
    /// Movements manually marked ignored (RF-10).
    pub ignored: Vec<MovementResponse>,
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
            counterparty_name: t.counterparty_name,
            counterparty_type: t.counterparty_type,
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
        state.counterparty_suggestions.clone(),
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
        state.deleted_audits.clone(),
        SuggestionPorts {
            batches: state.batches.clone(),
            accounts: state.ff3_accounts.clone(),
            suggestions: state.counterparty_suggestions.clone(),
        },
    );
    let result = uc.execute(BatchId::new(id)).await.map_err(ApiError)?;
    let mut suggestions = result.suggestions;

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
        missing: result
            .missing
            .into_iter()
            .map(|m| MissingMovementResponse {
                suggested_counterparty: suggestions.remove(&m.id).map(Into::into),
                movement: m.into(),
            })
            .collect(),
        ignored: result.ignored.into_iter().map(Into::into).collect(),
        extra: result.extra.into_iter().map(Into::into).collect(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct IgnoreMovementRequest {
    /// `true` = ignore, `false` = un-ignore (restore latest-run status).
    pub ignored: bool,
}

#[utoipa::path(
    post,
    path = "/api/imports/{id}/movements/{mid}/ignore",
    request_body = IgnoreMovementRequest,
    params(
        ("id" = i64, Path, description = "Batch id"),
        ("mid" = i64, Path, description = "Movement id"),
    ),
    responses(
        (status = 200, description = "Ignored state set", body = IgnoreMovementResponse),
        (status = 404, description = "Batch or movement not found"),
    ),
    tag = "reconcile",
)]
pub async fn ignore_movement(
    State(state): State<AppState>,
    Path((id, mid)): Path<(i64, i64)>,
    Json(body): Json<IgnoreMovementRequest>,
) -> Result<Json<IgnoreMovementResponse>, ApiError> {
    let uc = IgnoreMovement::new(
        state.movements.clone(),
        state.batches.clone(),
        state.reconciliations.clone(),
        state.matches.clone(),
    );
    let status = uc
        .execute(BatchId::new(id), MovementId::new(mid), body.ignored)
        .await
        .map_err(ApiError)?;

    Ok(Json(IgnoreMovementResponse {
        movement_id: mid,
        status: status.as_str().to_string(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct AttachmentRequest {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ApplyMovementBody {
    pub movement_id: i64,
    /// Existing counterparty account (external FF3 id): expense, revenue,
    /// liability or own asset. The transaction type is derived from it.
    /// Both this and `counterparty_name` absent = Cash; both present = failed.
    #[serde(default)]
    pub counterparty_ff3_id: Option<i64>,
    /// New expense (money out) or revenue (money in) account, created by name.
    #[serde(default)]
    pub counterparty_name: Option<String>,
    /// `true` = Cash chosen explicitly (RF-08 v0.11): booked like no
    /// counterparty, and remembered for the next suggestion. Sending nothing
    /// is also Cash but is not remembered. Fails with an id or a name.
    #[serde(default)]
    pub counterparty_cash: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub budget: Option<String>,
    /// Absent/`null` = the default creation tag (RF-19) is applied; a list,
    /// even `[]`, is sent to Firefly III as-is and replaces the default (RF-08).
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub attachment: Option<AttachmentRequest>,
}

#[derive(Deserialize, ToSchema)]
pub struct ApplyRequest {
    /// Selected movements with their resolved customization (v0.7).
    pub movements: Vec<ApplyMovementBody>,
}

#[derive(Serialize, ToSchema)]
pub struct ApplyReportResponse {
    pub movement_id: i64,
    /// One of `created`, `already_created`, `failed`.
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ff3_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// `withdrawal`, `deposit` or `transfer`; null unless created in this run.
    #[schema(example = "transfer")]
    pub transaction_type: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ApplyResponse {
    pub reports: Vec<ApplyReportResponse>,
    pub created_count: usize,
    pub already_created_count: usize,
    pub failed_count: usize,
    /// Reconciliation of the incremental confirmation run (null when
    /// nothing was created).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconciliation_id: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/imports/{id}/apply",
    request_body = ApplyRequest,
    params(("id" = i64, Path, description = "Batch id")),
    responses(
        (status = 200, description = "Apply report per movement", body = ApplyResponse),
        (status = 400, description = "Selection not in missing state, or batch not reconciled"),
        (status = 404, description = "Batch or movement not found"),
        (status = 502, description = "Firefly III unreachable (per-movement failures do not fail the batch)"),
    ),
    tag = "reconcile",
)]
pub async fn apply(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ApplyRequest>,
) -> Result<Json<ApplyResponse>, ApiError> {
    let _guard = state.apply_lock.lock().await;
    let uc = ApplyMissing::new(
        state.firefly.clone(),
        state.ff3_accounts.clone(),
        state.batches.clone(),
        state.movements.clone(),
        state.creations.clone(),
        RunPorts {
            reconciliations: state.reconciliations.clone(),
            matches: state.matches.clone(),
            ff3_transactions: state.ff3_transactions.clone(),
        },
        state.app_settings.clone(),
        state.counterparty_suggestions.clone(),
    );
    let result = uc
        .execute(ApplyMissingInput {
            batch_id: BatchId::new(id),
            movements: body
                .movements
                .into_iter()
                .map(|m| MovementApply {
                    movement_id: MovementId::new(m.movement_id),
                    counterparty_ff3_id: m.counterparty_ff3_id,
                    counterparty_name: m.counterparty_name,
                    counterparty_cash: m.counterparty_cash,
                    description: m.description,
                    category: m.category,
                    budget: m.budget,
                    tags: m.tags,
                    notes: m.notes,
                    attachment: m.attachment.map(|a| AttachmentPayload {
                        filename: a.filename,
                        content_base64: a.content_base64,
                    }),
                })
                .collect(),
        })
        .await
        .map_err(ApiError)?;

    Ok(Json(ApplyResponse {
        reports: result
            .reports
            .into_iter()
            .map(|r| {
                let outcome = match r.outcome {
                    ApplyOutcome::Created => "created",
                    ApplyOutcome::AlreadyCreated => "already_created",
                    ApplyOutcome::Failed => "failed",
                };
                ApplyReportResponse {
                    movement_id: r.movement_id.as_i64(),
                    outcome: outcome.to_string(),
                    ff3_id: r.ff3_id,
                    reason: r.reason,
                    transaction_type: r.transaction_type.map(|k| k.as_str().to_string()),
                }
            })
            .collect(),
        created_count: result.created_count,
        already_created_count: result.already_created_count,
        failed_count: result.failed_count,
        reconciliation_id: result.reconciliation_id.map(|rid| rid.as_i64()),
    }))
}
