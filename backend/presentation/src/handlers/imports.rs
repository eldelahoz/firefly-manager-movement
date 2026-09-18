//! Import (CSV upload) handlers.

use axum::extract::{Multipart, Path, State};
use axum::Json;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use utoipa::ToSchema;

use ffm_application::csv::ColumnMapping;
use ffm_application::ports::MovementRepository;
use ffm_application::use_cases::delete_import::DeleteImport;
use ffm_application::use_cases::get_import::GetImport;
use ffm_application::use_cases::list_imports::ListImports;
use ffm_application::use_cases::save_csv_mapping::SaveCsvMapping;
use ffm_application::use_cases::upload_import::{UploadImport, UploadImportInput};
use ffm_domain::entities::Ff3Account;
use ffm_domain::value_objects::DateFormat;
use std::collections::HashMap;

use crate::error::ApiError;
use crate::state::AppState;

/// Validates a wire date-format string (RF-02 v0.8). Blank/absent = None;
/// unknown values yield 400.
fn validate_date_format(raw: Option<&str>) -> Result<Option<DateFormat>, ApiError> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => DateFormat::from_str(s).map(Some).map_err(|_| {
            ApiError(ffm_application::error::ApplicationError::Parse(format!(
                "unknown date format '{s}'"
            )))
        }),
    }
}

#[derive(Serialize, ToSchema)]
pub struct ColumnMappingResponse {
    pub date_column: String,
    pub amount_column: String,
    pub description_column: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_account_column: Option<String>,
    /// Explicit date format used for parsing (RF-02 v0.8); absent = auto-try.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_format: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ParsedMovementResponse {
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
}

#[derive(Serialize, ToSchema)]
pub struct InvalidLineResponse {
    pub line_no: i32,
    pub reason: String,
    pub raw: String,
}

#[derive(Serialize, ToSchema)]
pub struct UploadImportResponse {
    pub batch_id: i64,
    pub filename: String,
    pub file_hash: String,
    pub headers: Vec<String>,
    pub detected_delimiter: String,
    pub total_rows: usize,
    pub valid_count: usize,
    pub invalid_count: usize,
    pub mapping_used: Option<ColumnMappingResponse>,
    pub valid: Vec<ParsedMovementResponse>,
    pub invalid: Vec<InvalidLineResponse>,
    pub origin_account_values: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BatchResponse {
    pub id: i64,
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub account_id: Option<i64>,
    pub account_name: Option<String>,
    pub movements_count: i64,
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
pub struct GetImportResponse {
    pub batch: BatchResponse,
    pub movements: Vec<MovementResponse>,
    /// CSV header row persisted at upload — used to resume the mapping step (RF-02, RF-18).
    pub headers: Vec<String>,
    /// Delimiter detected at upload time.
    pub detected_delimiter: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SaveMappingRequest {
    pub headers: Vec<String>,
    pub date_column: String,
    pub amount_column: String,
    pub description_column: String,
    #[serde(default)]
    pub origin_account_column: Option<String>,
    /// Memorized date format (RF-02 v0.8); absent/blank = no explicit format.
    #[serde(default)]
    pub date_format: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SaveMappingResponse {
    pub mapping_id: i64,
}

#[utoipa::path(
    post,
    path = "/api/imports",
    responses(
        (status = 200, description = "CSV uploaded and parsed", body = UploadImportResponse),
        (status = 400, description = "Invalid multipart request"),
        (status = 409, description = "Filename already imported"),
        (status = 413, description = "File too large"),
    ),
    tag = "imports",
)]
pub async fn upload_import(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadImportResponse>, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut currency: Option<String> = None;
    let mut account_id_field: Option<String> = None;

    let mut date_column: Option<String> = None;
    let mut amount_column: Option<String> = None;
    let mut description_column: Option<String> = None;
    let mut origin_account_column: Option<String> = None;
    let mut date_format_raw: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError(ffm_application::error::ApplicationError::Parse(
            e.to_string(),
        ))
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|e| {
                    ApiError(ffm_application::error::ApplicationError::Parse(
                        e.to_string(),
                    ))
                })?;
                file_bytes = Some(bytes.to_vec());
            }
            "currency" => {
                let text = field.text().await.map_err(|e| {
                    ApiError(ffm_application::error::ApplicationError::Parse(
                        e.to_string(),
                    ))
                })?;
                currency = Some(text.trim().to_string());
            }
            "date_column" => {
                date_column = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| {
                            ApiError(ffm_application::error::ApplicationError::Parse(
                                e.to_string(),
                            ))
                        })?
                        .trim()
                        .to_string(),
                );
            }
            "amount_column" => {
                amount_column = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| {
                            ApiError(ffm_application::error::ApplicationError::Parse(
                                e.to_string(),
                            ))
                        })?
                        .trim()
                        .to_string(),
                );
            }
            "description_column" => {
                description_column = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| {
                            ApiError(ffm_application::error::ApplicationError::Parse(
                                e.to_string(),
                            ))
                        })?
                        .trim()
                        .to_string(),
                );
            }
            "origin_account_column" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| {
                        ApiError(ffm_application::error::ApplicationError::Parse(
                            e.to_string(),
                        ))
                    })?
                    .trim()
                    .to_string();
                if !text.is_empty() {
                    origin_account_column = Some(text);
                }
            }
            "account_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| {
                        ApiError(ffm_application::error::ApplicationError::Parse(
                            e.to_string(),
                        ))
                    })?
                    .trim()
                    .to_string();
                if !text.is_empty() {
                    account_id_field = Some(text);
                }
            }
            "date_format" => {
                date_format_raw = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| {
                            ApiError(ffm_application::error::ApplicationError::Parse(
                                e.to_string(),
                            ))
                        })?
                        .trim()
                        .to_string(),
                );
            }
            _ => {
                // ignore unknown fields
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        ApiError(ffm_application::error::ApplicationError::Parse(
            "missing multipart field 'file'".into(),
        ))
    })?;
    let filename = filename.unwrap_or_else(|| "upload.csv".into());
    let currency = currency.unwrap_or_else(|| "EUR".into());
    let date_format = validate_date_format(date_format_raw.as_deref())?;

    // The frontend sends the FF3 account id; resolve it to the local
    // `ff3_accounts.id` stored on the batch (RF-04 v0.8: account locked at upload).
    let account_id = match account_id_field {
        Some(text) => {
            let ff3_id: i64 = text.parse().map_err(|_| {
                ApiError(ffm_application::error::ApplicationError::Parse(format!(
                    "invalid account_id '{text}'"
                )))
            })?;
            let accounts = state.ff3_accounts.all_asset().await.map_err(ApiError)?;
            let account = accounts.into_iter().find(|a| a.ff3_id == ff3_id);
            Some(account.map(|a| a.id).ok_or_else(|| {
                ApiError(ffm_application::error::ApplicationError::Parse(format!(
                    "unknown account {ff3_id}"
                )))
            })?)
        }
        None => None,
    };

    let mapping = match (date_column, amount_column, description_column) {
        (Some(d), Some(a), Some(desc)) => Some(ColumnMapping {
            date_column: d,
            amount_column: a,
            description_column: desc,
            origin_account_column,
            date_format,
        }),
        _ => None,
    };

    let uc = UploadImport::new(
        state.csv_decoder.clone(),
        state.batches.clone(),
        state.movements.clone(),
        state.csv_mappings.clone(),
        state.app_settings.clone(),
    );

    let result = uc
        .execute(UploadImportInput {
            filename: filename.clone(),
            bytes,
            currency,
            mapping,
            account_id,
        })
        .await
        .map_err(ApiError)?;

    Ok(Json(UploadImportResponse {
        batch_id: result.batch_id.as_i64(),
        filename: result.filename,
        file_hash: result.file_hash,
        headers: result.headers,
        detected_delimiter: result.detected_delimiter.to_string(),
        total_rows: result.total_rows,
        valid_count: result.valid_count,
        invalid_count: result.invalid_count,
        mapping_used: result.mapping_used.map(|m| ColumnMappingResponse {
            date_column: m.date_column,
            amount_column: m.amount_column,
            description_column: m.description_column,
            origin_account_column: m.origin_account_column,
            date_format: m.date_format.map(|f| f.as_str().to_string()),
        }),
        valid: result
            .valid
            .into_iter()
            .map(|m| ParsedMovementResponse {
                line_no: m.line_no,
                date: m.date,
                amount_cents: m.amount_cents.as_i64(),
                description: m.description,
            })
            .collect(),
        invalid: result
            .invalid
            .into_iter()
            .map(|l| InvalidLineResponse {
                line_no: l.line_no,
                reason: l.reason,
                raw: l.raw,
            })
            .collect(),
        origin_account_values: result.origin_account_values,
    }))
}

/// Index cached FF3 accounts by their local primary key, so a batch's
/// stored `account_id` (a local `ff3_accounts.id`) can be resolved to the
/// external FF3 id and name that the frontend accounts list uses as its key.
async fn account_index(
    repo: std::sync::Arc<dyn ffm_application::ports::Ff3AccountRepository>,
) -> Result<HashMap<i64, Ff3Account>, ApiError> {
    let accounts = repo.all_asset().await.map_err(ApiError)?;
    Ok(accounts.into_iter().map(|a| (a.id.as_i64(), a)).collect())
}

#[utoipa::path(
    get,
    path = "/api/imports",
    responses(
        (status = 200, description = "List of imports", body = [BatchResponse]),
    ),
    tag = "imports",
)]
pub async fn list_imports(
    State(state): State<AppState>,
) -> Result<Json<Vec<BatchResponse>>, ApiError> {
    let uc = ListImports::new(state.batches.clone());
    let batches = uc.execute().await.map_err(ApiError)?;
    let counts = state.movements.count_by_batch().await.map_err(ApiError)?;
    let account_by_local_id = account_index(state.ff3_accounts.clone()).await?;
    Ok(Json(
        batches
            .into_iter()
            .map(|b| {
                let account = b
                    .account_id
                    .and_then(|id| account_by_local_id.get(&id.as_i64()).cloned());
                BatchResponse {
                    id: b.id.as_i64(),
                    filename: b.filename,
                    file_hash: b.file_hash,
                    format: b.format,
                    currency: b.currency,
                    status: b.status.as_str().to_string(),
                    created_at: b.created_at,
                    account_id: account.as_ref().map(|a| a.ff3_id),
                    account_name: account.as_ref().map(|a| a.name.clone()),
                    movements_count: *counts.get(&b.id).unwrap_or(&0),
                }
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/imports/{id}",
    params(("id" = i64, Path, description = "Batch id")),
    responses(
        (status = 200, description = "Import detail", body = GetImportResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "imports",
)]
pub async fn get_import(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<GetImportResponse>, ApiError> {
    let uc = GetImport::new(state.batches.clone(), state.movements.clone());
    let (batch, movements) = uc
        .execute(ffm_domain::ids::BatchId::new(id))
        .await
        .map_err(ApiError)?;
    let account_by_local_id = account_index(state.ff3_accounts.clone()).await?;
    let account = batch
        .account_id
        .and_then(|a| account_by_local_id.get(&a.as_i64()).cloned());
    Ok(Json(GetImportResponse {
        batch: BatchResponse {
            id: batch.id.as_i64(),
            filename: batch.filename,
            file_hash: batch.file_hash,
            format: batch.format,
            currency: batch.currency,
            status: batch.status.as_str().to_string(),
            created_at: batch.created_at,
            account_id: account.as_ref().map(|a| a.ff3_id),
            account_name: account.as_ref().map(|a| a.name.clone()),
            movements_count: movements.len() as i64,
        },
        movements: movements
            .into_iter()
            .map(|m| MovementResponse {
                id: m.id.as_i64(),
                line_no: m.line_no,
                date: m.date,
                amount_cents: m.amount_cents.as_i64(),
                description: m.description,
                status: m.status.as_str().to_string(),
            })
            .collect(),
        headers: batch.headers,
        detected_delimiter: batch.detected_delimiter,
    }))
}

#[derive(Serialize, ToSchema)]
pub struct DeleteImportResponse {
    pub id: i64,
    pub status: String,
}

#[utoipa::path(
    delete,
    path = "/api/imports/{id}",
    params(("id" = i64, Path, description = "Batch id")),
    responses(
        (status = 200, description = "Import deleted", body = DeleteImportResponse),
        (status = 400, description = "Batch is not in 'parsed' state"),
        (status = 404, description = "Not found"),
    ),
    tag = "imports",
)]
pub async fn delete_import(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteImportResponse>, ApiError> {
    let uc = DeleteImport::new(state.batches.clone(), state.movements.clone());
    uc.execute(ffm_domain::ids::BatchId::new(id))
        .await
        .map_err(ApiError)?;
    Ok(Json(DeleteImportResponse {
        id,
        status: "deleted".to_string(),
    }))
}

#[utoipa::path(
    put,
    path = "/api/imports/{id}/mapping",
    request_body = SaveMappingRequest,
    params(("id" = i64, Path, description = "Batch id")),
    responses(
        (status = 200, description = "Mapping saved", body = SaveMappingResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "imports",
)]
pub async fn save_mapping(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<SaveMappingRequest>,
) -> Result<Json<SaveMappingResponse>, ApiError> {
    // Ensure the batch exists (route is batch-scoped).
    state
        .batches
        .get(ffm_domain::ids::BatchId::new(id))
        .await
        .map_err(ApiError)?;

    let uc = SaveCsvMapping::new(state.csv_mappings.clone());
    let date_format = validate_date_format(body.date_format.as_deref())?;
    let mapping_id = uc
        .execute(
            body.headers,
            ColumnMapping {
                date_column: body.date_column,
                amount_column: body.amount_column,
                description_column: body.description_column,
                origin_account_column: body.origin_account_column,
                date_format,
            },
        )
        .await
        .map_err(ApiError)?;

    Ok(Json(SaveMappingResponse {
        mapping_id: mapping_id.as_i64(),
    }))
}
