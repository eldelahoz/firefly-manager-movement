//! CSV parsing and column mapping (pure application logic).
//!
//! Detects the delimiter, maps columns to domain fields, validates each row
//! (date + amount parsing), and returns valid movements alongside invalid
//! lines with their reason.

use std::io::Cursor;
use std::str::FromStr;

use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map};

use ffm_domain::value_objects::AmountCents;

use crate::error::ApplicationError;

/// Maps CSV column names to domain fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub date_column: String,
    pub amount_column: String,
    pub description_column: String,
}

/// A successfully parsed movement (before DB insert, no IDs yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMovement {
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
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
}

/// Detects the most likely delimiter from the first non-empty line.
pub fn detect_delimiter(content: &str) -> char {
    let first_line = content.lines().next().unwrap_or("");
    let commas = first_line.matches(',').count();
    let semicolons = first_line.matches(';').count();
    let tabs = first_line.matches('\t').count();

    let mut best = ',';
    let mut best_count = commas;
    if semicolons > best_count {
        best = ';';
        best_count = semicolons;
    }
    if tabs > best_count {
        best = '\t';
    }
    best
}

/// Tries to auto-detect the column mapping from header names.
/// Returns `None` if any of the three columns cannot be guessed.
pub fn auto_detect_mapping(headers: &[String]) -> Option<ColumnMapping> {
    let date_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("date") || l.contains("fecha")
        })
        .cloned()?;

    let amount_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("amount")
                || l.contains("montant")
                || l.contains("importe")
                || l.contains("valor")
                || l.contains("montante")
                || l.contains("value")
        })
        .cloned()?;

    let description_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("description")
                || l.contains("descrip")
                || l.contains("libell")
                || l.contains("label")
                || l.contains("memo")
                || l.contains("detail")
                || l.contains("narrative")
        })
        .cloned()?;

    Some(ColumnMapping {
        date_column,
        amount_column,
        description_column,
    })
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

    let mut valid = Vec::new();
    let mut invalid = Vec::new();
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

        let date = match parse_date(date_str) {
            Some(d) => d,
            None => {
                invalid.push(InvalidLine {
                    line_no,
                    reason: format!("invalid date '{}'", date_str),
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
        });
        line_no += 1;
    }

    Ok(CsvParseResult {
        headers,
        detected_delimiter: delimiter,
        valid,
        invalid,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delimiter_comma() {
        assert_eq!(detect_delimiter("a,b,c\n1,2,3"), ',');
    }

    #[test]
    fn delimiter_semicolon() {
        assert_eq!(detect_delimiter("a;b;c\n1;2;3"), ';');
    }

    #[test]
    fn delimiter_tab() {
        assert_eq!(detect_delimiter("a\tb\tc\n1\t2\t3"), '\t');
    }

    #[test]
    fn auto_detect_english() {
        let h = vec!["Date".into(), "Amount".into(), "Description".into()];
        let m = auto_detect_mapping(&h).unwrap();
        assert_eq!(m.date_column, "Date");
        assert_eq!(m.amount_column, "Amount");
        assert_eq!(m.description_column, "Description");
    }

    #[test]
    fn auto_detect_french() {
        let h = vec!["Date".into(), "Libellé".into(), "Montant".into()];
        let m = auto_detect_mapping(&h).unwrap();
        assert_eq!(m.date_column, "Date");
        assert_eq!(m.amount_column, "Montant");
        assert_eq!(m.description_column, "Libellé");
    }

    #[test]
    fn auto_detect_none_for_unknown() {
        let h = vec!["col1".into(), "col2".into(), "col3".into()];
        assert!(auto_detect_mapping(&h).is_none());
    }

    #[test]
    fn parse_valid_comma() {
        let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n2026-01-06,-50.00,Lunch";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        assert_eq!(r.valid.len(), 2);
        assert!(r.invalid.is_empty());
        assert_eq!(r.valid[0].amount_cents.as_i64(), 10000);
        assert_eq!(r.valid[1].amount_cents.as_i64(), -5000);
    }

    #[test]
    fn parse_invalid_date() {
        let csv = "Date,Amount,Description\nbad,100.00,Coffee";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        assert_eq!(r.valid.len(), 0);
        assert_eq!(r.invalid.len(), 1);
        assert!(r.invalid[0].reason.contains("date"));
    }

    #[test]
    fn parse_invalid_amount() {
        let csv = "Date,Amount,Description\n2026-01-05,abc,Coffee";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        assert_eq!(r.valid.len(), 0);
        assert_eq!(r.invalid.len(), 1);
        assert!(r.invalid[0].reason.contains("amount"));
    }

    #[test]
    fn parse_european_date() {
        let csv = "Date,Amount,Description\n05/01/2026,100.00,Coffee";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        assert_eq!(r.valid.len(), 1);
        assert_eq!(
            r.valid[0].date,
            NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
        );
    }

    #[test]
    fn parse_raw_json_has_fields() {
        let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        let raw: serde_json::Value = serde_json::from_str(&r.valid[0].raw_json).unwrap();
        assert_eq!(raw["Date"], "2026-01-05");
        assert_eq!(raw["Amount"], "100.00");
        assert_eq!(raw["Description"], "Coffee");
    }

    #[test]
    fn parse_empty_csv() {
        let csv = "Date,Amount,Description";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        assert!(r.valid.is_empty());
        assert!(r.invalid.is_empty());
        assert_eq!(r.headers.len(), 3);
    }

    #[test]
    fn parse_skips_empty_lines() {
        let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n\n2026-01-06,-50.00,Lunch";
        let m = ColumnMapping {
            date_column: "Date".into(),
            amount_column: "Amount".into(),
            description_column: "Description".into(),
        };
        let r = parse_csv(csv, &m).unwrap();
        assert_eq!(r.valid.len(), 2);
        assert!(r.invalid.is_empty());
    }
}
