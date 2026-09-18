//! UploadImport use case.
//! Decodes raw bytes, parses CSV, validates lines (RF-03), inserts batch + movements.

use sha2::{Digest, Sha256};

use ffm_domain::ids::BatchId;
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};

use crate::csv::{auto_detect_mapping, parse_csv, ColumnMapping, InvalidLine, ParsedMovement};
use crate::dtos::{NewBatch, NewMovement};
use crate::error::ApplicationError;
use crate::ports::{
    AppSettingsRepository, BatchRepository, CsvDecoder, CsvMappingRepository, MovementRepository,
};

/// Input for the upload import use case.
pub struct UploadImportInput {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub currency: String,
    pub mapping: Option<ColumnMapping>,
}

/// Result of the upload import use case.
pub struct UploadImportResult {
    pub batch_id: BatchId,
    pub filename: String,
    pub file_hash: String,
    pub headers: Vec<String>,
    pub detected_delimiter: char,
    /// Total number of non-empty data rows found (excluding header).
    pub total_rows: usize,
    /// Total number of valid rows (may be > `valid.len()` because `valid` is a preview).
    pub valid_count: usize,
    /// Total number of invalid rows (may be > `invalid.len()` because `invalid` is a preview).
    pub invalid_count: usize,
    pub mapping_used: Option<ColumnMapping>,
    /// Preview of parsed valid rows (capped).
    pub valid: Vec<ParsedMovement>,
    /// Preview of invalid rows (capped).
    pub invalid: Vec<InvalidLine>,
    /// Distinct origin-account values extracted from the CSV (sorted, deduplicated).
    pub origin_account_values: Vec<String>,
}

const PREVIEW_LIMIT: usize = 50;

pub struct UploadImport<D, B, M, C, S> {
    decoder: D,
    batches: B,
    movements: M,
    csv_mappings: C,
    settings: S,
}

impl<D, B, M, C, S> UploadImport<D, B, M, C, S> {
    pub fn new(decoder: D, batches: B, movements: M, csv_mappings: C, settings: S) -> Self {
        Self {
            decoder,
            batches,
            movements,
            csv_mappings,
            settings,
        }
    }
}

impl<D, B, M, C, S> UploadImport<D, B, M, C, S>
where
    D: CsvDecoder,
    B: BatchRepository,
    M: MovementRepository,
    C: CsvMappingRepository,
    S: AppSettingsRepository,
{
    pub async fn execute(
        &self,
        input: UploadImportInput,
    ) -> Result<UploadImportResult, ApplicationError> {
        let existing = self.batches.find_by_filename(&input.filename).await?;
        if !existing.is_empty() {
            return Err(ApplicationError::Conflict(format!(
                "ya existe un import con el archivo '{}'",
                input.filename
            )));
        }

        let file_hash = compute_hash(&input.bytes);

        let content = self.decoder.decode(input.bytes).await?;

        let delimiter = crate::csv::detect_delimiter(&content);

        let headers = extract_headers(&content, delimiter);

        let signature = HeaderSignature::from_headers(&headers);

        let mut mapping = if let Some(m) = input.mapping {
            Some(m)
        } else if let Some(saved) = self.csv_mappings.by_signature(signature).await? {
            let mapping: ColumnMapping = serde_json::from_str(&saved.mapping_json)
                .map_err(|e| ApplicationError::Parse(format!("invalid saved mapping JSON: {e}")))?;
            Some(mapping)
        } else {
            auto_detect_mapping(&headers)
        };

        // Date-format precedence (RF-02 v0.8): explicit (multipart or
        // memorized signature) > global settings default > legacy auto-try.
        if let Some(ref mut m) = mapping {
            if m.date_format.is_none() {
                if let Some(settings) = self.settings.get().await? {
                    m.date_format = settings.default_date_format;
                }
            }
        }

        let batch_id = self
            .batches
            .insert(NewBatch {
                filename: input.filename.clone(),
                file_hash: file_hash.clone(),
                format: "csv".to_string(),
                currency: input.currency,
                account_id: None,
                status: BatchStatus::Parsed,
                headers: headers.clone(),
                detected_delimiter: delimiter.to_string(),
            })
            .await?;

        if let Some(ref mapping) = mapping {
            let parse_result = parse_csv(&content, mapping)?;

            let total_rows = parse_result.valid.len() + parse_result.invalid.len();
            let valid_count = parse_result.valid.len();
            let invalid_count = parse_result.invalid.len();

            let new_movements: Vec<NewMovement> = parse_result
                .valid
                .iter()
                .map(|m| NewMovement {
                    batch_id,
                    line_no: m.line_no,
                    date: m.date,
                    amount_cents: m.amount_cents,
                    description: m.description.clone(),
                    raw_json: m.raw_json.clone(),
                    status: MovementStatus::Imported,
                })
                .collect();

            if !new_movements.is_empty() {
                self.movements.insert_batch(new_movements).await?;
            }

            return Ok(UploadImportResult {
                batch_id,
                filename: input.filename,
                file_hash,
                headers: parse_result.headers,
                detected_delimiter: parse_result.detected_delimiter,
                total_rows,
                valid_count,
                invalid_count,
                mapping_used: Some(mapping.clone()),
                valid: parse_result.valid.into_iter().take(PREVIEW_LIMIT).collect(),
                invalid: parse_result
                    .invalid
                    .into_iter()
                    .take(PREVIEW_LIMIT)
                    .collect(),
                origin_account_values: parse_result.origin_account_values,
            });
        }

        let total_rows = count_rows(&content, delimiter)?;

        Ok(UploadImportResult {
            batch_id,
            filename: input.filename,
            file_hash,
            headers,
            detected_delimiter: delimiter,
            total_rows,
            valid_count: 0,
            invalid_count: 0,
            mapping_used: None,
            valid: Vec::new(),
            invalid: Vec::new(),
            origin_account_values: Vec::new(),
        })
    }
}

fn compute_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

fn extract_headers(content: &str, delimiter: char) -> Vec<String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(true)
        .flexible(true)
        .from_reader(std::io::Cursor::new(content));

    reader
        .headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

fn count_rows(content: &str, delimiter: char) -> Result<usize, ApplicationError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(true)
        .flexible(true)
        .from_reader(std::io::Cursor::new(content));

    let mut count = 0usize;
    for result in reader.records() {
        let record = result
            .map_err(|e| ApplicationError::Parse(format!("failed to read CSV record: {e}")))?;
        if record.iter().all(|v| v.trim().is_empty()) {
            continue;
        }
        count += 1;
    }
    Ok(count)
}
