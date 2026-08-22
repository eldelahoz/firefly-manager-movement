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
    };
    let result = parse_csv(csv, &mapping).unwrap();
    assert_eq!(result.valid.len(), 2);
    assert!(result.invalid.is_empty());
}
