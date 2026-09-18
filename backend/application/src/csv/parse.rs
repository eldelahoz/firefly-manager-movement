use std::io::Cursor;
use std::str::FromStr;

use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde_json::{json, Map};

use ffm_domain::value_objects::AmountCents;
use ffm_domain::value_objects::DateFormat;

use super::detect::detect_delimiter;
use super::mapping::ColumnMapping;
use crate::error::ApplicationError;

/// A successfully parsed movement (before DB insert, no IDs yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMovement {
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub origin_account: Option<String>,
}

/// A line that failed validation, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidLine {
    pub line_no: i32,
    pub reason: String,
    pub raw: String,
}

/// Result of parsing a CSV file.
#[derive(Debug, Clone)]
pub struct CsvParseResult {
    pub headers: Vec<String>,
    pub detected_delimiter: char,
    pub valid: Vec<ParsedMovement>,
    pub invalid: Vec<InvalidLine>,
    /// Distinct non-empty origin-account values extracted from the CSV, sorted.
    pub origin_account_values: Vec<String>,
}

/// Parses a decoded CSV string using the given column mapping.
///
/// Each row is validated: date and amount must parse successfully.
/// Invalid rows are kept in `invalid` with a reason.
pub fn parse_csv(
    content: &str,
    mapping: &ColumnMapping,
) -> Result<CsvParseResult, ApplicationError> {
    let delimiter = detect_delimiter(content);

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(true)
        .flexible(true)
        .from_reader(Cursor::new(content));

    let header_record = reader
        .headers()
        .map_err(|e| ApplicationError::Parse(format!("failed to read CSV headers: {e}")))?;

    let headers: Vec<String> = header_record.iter().map(|h| h.to_string()).collect();

    let date_idx = find_column_index(&headers, &mapping.date_column);
    let amount_idx = find_column_index(&headers, &mapping.amount_column);
    let desc_idx = find_column_index(&headers, &mapping.description_column);

    if date_idx.is_none() {
        return Err(ApplicationError::Parse(format!(
            "column '{}' not found in headers {:?}",
            mapping.date_column, headers
        )));
    }
    if amount_idx.is_none() {
        return Err(ApplicationError::Parse(format!(
            "column '{}' not found in headers {:?}",
            mapping.amount_column, headers
        )));
    }
    if desc_idx.is_none() {
        return Err(ApplicationError::Parse(format!(
            "column '{}' not found in headers {:?}",
            mapping.description_column, headers
        )));
    }

    let date_idx = date_idx.unwrap();
    let amount_idx = amount_idx.unwrap();
    let desc_idx = desc_idx.unwrap();

    let origin_account_idx = mapping
        .origin_account_column
        .as_ref()
        .and_then(|col| find_column_index(&headers, col));

    let mut valid = Vec::new();
    let mut invalid = Vec::new();
    let mut origin_set = std::collections::BTreeSet::new();
    let mut line_no = 1i32;

    for result in reader.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                invalid.push(InvalidLine {
                    line_no,
                    reason: format!("CSV parse error: {e}"),
                    raw: String::new(),
                });
                line_no += 1;
                continue;
            }
        };

        let raw_json = build_raw_json(&headers, &record);
        let raw_line = record.iter().collect::<Vec<_>>().join(",");

        let date_str = record.get(date_idx).unwrap_or("").trim();
        let amount_str = record.get(amount_idx).unwrap_or("").trim();
        let desc_str = record.get(desc_idx).unwrap_or("").trim();

        let date = match parse_date_in(mapping.date_format, date_str) {
            Some(d) => d,
            None => {
                let reason = match mapping.date_format {
                    Some(f) => format!("invalid date '{}' (expected format {})", date_str, f.as_str()),
                    None => format!("invalid date '{}'", date_str),
                };
                invalid.push(InvalidLine {
                    line_no,
                    reason,
                    raw: raw_line,
                });
                line_no += 1;
                continue;
            }
        };

        let amount = match AmountCents::from_str(amount_str) {
            Ok(a) => a,
            Err(e) => {
                invalid.push(InvalidLine {
                    line_no,
                    reason: format!("invalid amount '{}': {:?}", amount_str, e),
                    raw: raw_line,
                });
                line_no += 1;
                continue;
            }
        };

        valid.push(ParsedMovement {
            line_no,
            date,
            amount_cents: amount,
            description: desc_str.to_string(),
            raw_json,
            origin_account: origin_account_idx.and_then(|idx| {
                let val = record.get(idx)?.trim().to_string();
                if val.is_empty() {
                    None
                } else {
                    origin_set.insert(val.clone());
                    Some(val)
                }
            }),
        });
        line_no += 1;
    }

    Ok(CsvParseResult {
        headers,
        detected_delimiter: delimiter,
        valid,
        invalid,
        origin_account_values: origin_set.into_iter().collect(),
    })
}

fn find_column_index(headers: &[String], column_name: &str) -> Option<usize> {
    let target = column_name.trim().to_lowercase();
    headers
        .iter()
        .position(|h| h.trim().to_lowercase() == target)
}

fn build_raw_json(headers: &[String], record: &csv::StringRecord) -> String {
    let mut map = Map::new();
    for (i, header) in headers.iter().enumerate() {
        if let Some(val) = record.get(i) {
            map.insert(header.clone(), json!(val));
        }
    }
    json!(map).to_string()
}

/// Tries common date formats used in bank CSVs.
fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    const FORMATS: &[&str] = &[
        "%Y-%m-%d", "%d/%m/%Y", "%d.%m.%Y", "%d-%m-%Y", "%m/%d/%Y", "%Y/%m/%d",
    ];
    for fmt in FORMATS {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

/// Parses a date using ONLY the explicit format when one is set
/// (RF-02 v0.8); falls back to the legacy auto-try otherwise.
fn parse_date_in(explicit: Option<DateFormat>, s: &str) -> Option<NaiveDate> {
    match explicit {
        Some(f) => NaiveDate::parse_from_str(s.trim(), f.chrono_format()).ok(),
        None => parse_date(s),
    }
}
