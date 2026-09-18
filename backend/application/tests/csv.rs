use ffm_application::csv::{auto_detect_mapping, detect_delimiter, parse_csv, ColumnMapping};

// ── Delimiter detection ─────────────────────────────────────────────────

#[test]
fn detect_comma_delimiter() {
    let csv = "a,b,c\n1,2,3\n4,5,6";
    assert_eq!(detect_delimiter(csv), ',');
}

#[test]
fn detect_semicolon_delimiter() {
    let csv = "a;b;c\n1;2;3\n4;5;6";
    assert_eq!(detect_delimiter(csv), ';');
}

#[test]
fn detect_tab_delimiter() {
    let csv = "a\tb\tc\n1\t2\t3\n4\t5\t6";
    assert_eq!(detect_delimiter(csv), '\t');
}

// ── Auto-detect mapping ─────────────────────────────────────────────────

#[test]
fn auto_detect_english_headers() {
    let headers = vec![
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
    ];
    let mapping = auto_detect_mapping(&headers).unwrap();
    assert_eq!(mapping.date_column, "Date");
    assert_eq!(mapping.amount_column, "Amount");
    assert_eq!(mapping.description_column, "Description");
}

#[test]
fn auto_detect_french_headers() {
    let headers = vec![
        "Date".to_string(),
        "Libellé".to_string(),
        "Montant".to_string(),
    ];
    let mapping = auto_detect_mapping(&headers).unwrap();
    assert_eq!(mapping.date_column, "Date");
    assert_eq!(mapping.amount_column, "Montant");
    assert_eq!(mapping.description_column, "Libellé");
}

#[test]
fn auto_detect_returns_none_for_unknown_headers() {
    let headers = vec!["col1".to_string(), "col2".to_string(), "col3".to_string()];
    assert!(auto_detect_mapping(&headers).is_none());
}

#[test]
fn auto_detect_spanish_headers() {
    let headers = vec![
        "Fecha".to_string(),
        "Descripción".to_string(),
        "Referencia".to_string(),
        "Valor".to_string(),
    ];
    let mapping = auto_detect_mapping(&headers).unwrap();
    assert_eq!(mapping.date_column, "Fecha");
    assert_eq!(mapping.amount_column, "Valor");
    assert_eq!(mapping.description_column, "Descripción");
}

// ── Full parse ──────────────────────────────────────────────────────────

#[test]
fn parse_valid_csv_comma() {
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n2026-01-06,-50.00,Lunch";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.headers, vec!["Date", "Amount", "Description"]);
    assert_eq!(result.detected_delimiter, ',');
    assert_eq!(result.valid.len(), 2);
    assert!(result.invalid.is_empty());

    let m0 = &result.valid[0];
    assert_eq!(m0.line_no, 1);
    assert_eq!(
        m0.date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );
    assert_eq!(m0.amount_cents.as_i64(), 10000);
    assert_eq!(m0.description, "Coffee");

    let m1 = &result.valid[1];
    assert_eq!(m1.line_no, 2);
    assert_eq!(m1.amount_cents.as_i64(), -5000);
    assert_eq!(m1.description, "Lunch");
}

#[test]
fn parse_valid_csv_semicolon() {
    let csv = "Date;Montant;Libellé\n2026-01-05;100,00;Café\n2026-01-06;-50,00;Déjeuner";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Montant".to_string(),
        description_column: "Libellé".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.detected_delimiter, ';');
    assert_eq!(result.valid.len(), 2);
    assert_eq!(result.valid[0].amount_cents.as_i64(), 10000);
    assert_eq!(result.valid[1].amount_cents.as_i64(), -5000);
}

#[test]
fn parse_invalid_date_kept_in_invalid() {
    let csv = "Date,Amount,Description\nbad-date,100.00,Coffee\n2026-01-06,-50.00,Lunch";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 1);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].line_no, 1);
    assert!(result.invalid[0].reason.contains("date"));
}

#[test]
fn parse_invalid_amount_kept_in_invalid() {
    let csv = "Date,Amount,Description\n2026-01-05,abc,Coffee";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert!(result.invalid[0].reason.contains("amount"));
}

#[test]
fn parse_european_date_format() {
    let csv = "Date,Amount,Description\n05/01/2026,100.00,Coffee";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 1);
    assert_eq!(
        result.valid[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );
}

#[test]
fn parse_raw_json_contains_original_fields() {
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    let raw: serde_json::Value = serde_json::from_str(&result.valid[0].raw_json).unwrap();
    assert_eq!(raw["Date"], "2026-01-05");
    assert_eq!(raw["Amount"], "100.00");
    assert_eq!(raw["Description"], "Coffee");
}

#[test]
fn parse_empty_csv_returns_empty_result() {
    let csv = "Date,Amount,Description";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert!(result.valid.is_empty());
    assert!(result.invalid.is_empty());
    assert_eq!(result.headers.len(), 3);
}

#[test]
fn parse_skips_empty_lines() {
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee\n\n2026-01-06,-50.00,Lunch";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 2);
    assert!(result.invalid.is_empty());
}

// ── Origin-account column ────────────────────────────────────────────────

#[test]
fn auto_detect_origin_account_column() {
    let headers = vec![
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
        "Account".to_string(),
    ];
    let mapping = auto_detect_mapping(&headers).unwrap();
    assert_eq!(mapping.origin_account_column.as_deref(), Some("Account"));
}

#[test]
fn auto_detect_no_origin_account_column() {
    let headers = vec![
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
    ];
    let mapping = auto_detect_mapping(&headers).unwrap();
    assert!(mapping.origin_account_column.is_none());
}

#[test]
fn auto_detect_french_compte_origin() {
    let headers = vec![
        "Date".to_string(),
        "Montant".to_string(),
        "Libellé".to_string(),
        "Compte".to_string(),
    ];
    let mapping = auto_detect_mapping(&headers).unwrap();
    assert_eq!(mapping.origin_account_column.as_deref(), Some("Compte"));
}

#[test]
fn parse_with_origin_account_column() {
    let csv = "Date,Amount,Description,Account\n2026-01-05,100.00,Coffee,ACC-001\n2026-01-06,-50.00,Lunch,ACC-002\n2026-01-07,25.00,Tea,ACC-001";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: Some("Account".to_string()),
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 3);
    assert_eq!(result.valid[0].origin_account.as_deref(), Some("ACC-001"));
    assert_eq!(result.valid[1].origin_account.as_deref(), Some("ACC-002"));
    assert_eq!(result.valid[2].origin_account.as_deref(), Some("ACC-001"));
    // Distinct values sorted
    assert_eq!(result.origin_account_values, vec!["ACC-001", "ACC-002"]);
}

#[test]
fn parse_with_origin_account_empty_values() {
    let csv = "Date,Amount,Description,Account\n2026-01-05,100.00,Coffee,ACC-001\n2026-01-06,-50.00,Lunch,\n2026-01-07,25.00,Tea,ACC-001";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: Some("Account".to_string()),
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 3);
    assert_eq!(result.valid[0].origin_account.as_deref(), Some("ACC-001"));
    assert!(result.valid[1].origin_account.is_none());
    assert_eq!(result.valid[2].origin_account.as_deref(), Some("ACC-001"));
    // Empty value not included in distinct set
    assert_eq!(result.origin_account_values, vec!["ACC-001"]);
}

#[test]
fn parse_without_origin_account_column() {
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 1);
    assert!(result.valid[0].origin_account.is_none());
    assert!(result.origin_account_values.is_empty());
}

// #region Explicit date format (RF-02 v0.8)

use ffm_domain::value_objects::DateFormat;

fn mapping_with_format(date_format: DateFormat) -> ColumnMapping {
    ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: Some(date_format),
    }
}

#[test]
fn parse_csv_uses_explicit_format_exclusively() {
    let csv = "Date,Amount,Description\n05/01/2026,100.00,Coffee\n13/01/2026,-50.00,Lunch";
    let mapping = mapping_with_format(DateFormat::DdMmYyyySlash);
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 2);
    assert_eq!(
        result.valid[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );
    assert_eq!(
        result.valid[1].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 13).unwrap()
    );
}

#[test]
fn parse_csv_explicit_format_rejects_other_layouts() {
    let csv = "Date,Amount,Description\n2026-01-05,100.00,Coffee";
    let mapping = mapping_with_format(DateFormat::DdMmYyyySlash);
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 0);
    assert_eq!(result.invalid.len(), 1);
}

#[test]
fn parse_csv_invalid_date_names_expected_format() {
    let csv = "Date,Amount,Description\nbad,100.00,Coffee";
    let mapping = mapping_with_format(DateFormat::DdMmYyyySlash);
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(
        result.invalid[0].reason,
        "invalid date 'bad' (expected format dd/mm/yyyy)"
    );
}

#[test]
fn parse_csv_explicit_mm_dd_reads_first_component_as_month() {
    let csv = "Date,Amount,Description\n05/01/2026,100.00,Coffee";
    let mapping = mapping_with_format(DateFormat::MmDdYyyySlash);
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(
        result.valid[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
    );
}

#[test]
fn no_format_keeps_legacy_auto_try() {
    let csv = "Date,Amount,Description\n05/01/2026,100.00,Coffee\n2026-01-13,-50.00,Lunch";
    let mapping = ColumnMapping {
        date_column: "Date".to_string(),
        amount_column: "Amount".to_string(),
        description_column: "Description".to_string(),
        origin_account_column: None,
        date_format: None,
    };
    let result = parse_csv(csv, &mapping).unwrap();
    // Legacy: %d/%m/%Y tried before %m/%d/%Y — behavior unchanged.
    assert_eq!(
        result.valid[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );
    assert_eq!(
        result.valid[1].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 13).unwrap()
    );
}

#[test]
fn column_mapping_json_round_trips_date_format() {
    let mapping = mapping_with_format(DateFormat::YyyyMmDdDash);
    let json = serde_json::to_string(&mapping).unwrap();
    assert!(json.contains(r#""date_format":"yyyy-mm-dd""#));
    let back: ColumnMapping = serde_json::from_str(&json).unwrap();
    assert_eq!(back.date_format, Some(DateFormat::YyyyMmDdDash));
}
// #endregion
